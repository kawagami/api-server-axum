use sqlx::{Pool, Postgres};
use std::fmt;
use tokio::sync::mpsc;
use tracing::{field::{Field, Visit}, Event, Subscriber};
use tracing_subscriber::{layer::Context, Layer};

pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

struct MessageVisitor(String);

impl Visit for MessageVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_owned();
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
        }
    }
}

pub struct DbLogLayer {
    tx: mpsc::Sender<LogEntry>,
}

impl DbLogLayer {
    pub fn new(tx: mpsc::Sender<LogEntry>) -> Self {
        Self { tx }
    }
}

impl<S: Subscriber> Layer<S> for DbLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();

        // 只落地 WARN / ERROR（業務事件走 admin_audit_logs，不在此表）
        if *meta.level() > tracing::Level::WARN {
            return;
        }

        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);

        let entry = LogEntry {
            level: meta.level().to_string(),
            message: visitor.0,
            target: meta.target().to_owned(),
            file: meta.file().map(str::to_owned),
            line: meta.line(),
        };

        let _ = self.tx.try_send(entry);
    }
}

const BATCH_SIZE: usize = 50;
const FLUSH_INTERVAL_MS: u64 = 500;

pub async fn log_writer(mut rx: mpsc::Receiver<LogEntry>, pool: Pool<Postgres>) {
    let mut buf: Vec<LogEntry> = Vec::with_capacity(BATCH_SIZE);
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(FLUSH_INTERVAL_MS));

    loop {
        tokio::select! {
            entry = rx.recv() => {
                match entry {
                    Some(e) => {
                        buf.push(e);
                        if buf.len() >= BATCH_SIZE {
                            flush(&pool, &mut buf).await;
                        }
                    }
                    None => {
                        flush(&pool, &mut buf).await;
                        return;
                    }
                }
            }
            _ = interval.tick() => {
                if !buf.is_empty() {
                    flush(&pool, &mut buf).await;
                }
            }
        }
    }
}

async fn flush(pool: &Pool<Postgres>, buf: &mut Vec<LogEntry>) {
    let levels: Vec<&str> = buf.iter().map(|e| e.level.as_str()).collect();
    let messages: Vec<&str> = buf.iter().map(|e| e.message.as_str()).collect();
    let targets: Vec<&str> = buf.iter().map(|e| e.target.as_str()).collect();
    let files: Vec<Option<&str>> = buf.iter().map(|e| e.file.as_deref()).collect();
    let lines: Vec<Option<i32>> = buf.iter().map(|e| e.line.map(|l| l as i32)).collect();

    let _ = sqlx::query(
        "INSERT INTO logs (level, message, target, file, line)
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::int[])",
    )
    .bind(&levels)
    .bind(&messages)
    .bind(&targets)
    .bind(&files)
    .bind(&lines)
    .execute(pool)
    .await;

    buf.clear();
}
