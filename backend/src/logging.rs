use serde_json::{Map, Value};
use sqlx::{Pool, Postgres};
use std::fmt;
use tokio::sync::mpsc;
use tracing::{
    field::{Field, Visit},
    span, Event, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    /// 來自 `routes.rs` request span 的追蹤 id（非請求路徑的 log 為 None）
    pub request_id: Option<String>,
    /// event 與所屬 span 的其餘 field（`?self` 的錯誤細節、method、path…）
    pub fields: Option<Value>,
}

/// 單一 field 值的長度上限。`?self` 這類 Debug 輸出可以很長（含上游回應片段、
/// anyhow chain），而這張表跑在 1 核 1G 的機器上，截斷比整包放行安全。
const MAX_FIELD_LEN: usize = 2000;

/// 把 event / span 的 field 收成 JSON。
///
/// `message` 單獨抽出來當 log 正文，其餘原樣留在 map 裡 ——
/// `tracing::error!(?self, "System error occurred")` 的錯誤細節就在 `self` 這個 key，
/// 那是查線上 500 唯一能看到真正原因的地方。
#[derive(Default)]
struct FieldVisitor {
    message: String,
    fields: Map<String, Value>,
}

impl FieldVisitor {
    fn insert(&mut self, field: &Field, value: Value) {
        if field.name() == "message" {
            if let Value::String(s) = value {
                self.message = s;
            }
            return;
        }
        self.fields.insert(field.name().to_owned(), value);
    }
}

/// 超長字串截斷。走 `char_indices` 一次掃到上限就停，不先數完整個字串長度。
fn truncated(s: String) -> Value {
    match s.char_indices().nth(MAX_FIELD_LEN) {
        Some((idx, _)) => {
            let mut s = s;
            s.truncate(idx);
            s.push_str("…[truncated]");
            Value::String(s)
        }
        None => Value::String(s),
    }
}

impl Visit for FieldVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field, truncated(value.to_owned()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.insert(field, truncated(format!("{value:?}")));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field, Value::from(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field, Value::from(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field, Value::from(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.insert(field, Value::from(value));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.insert(field, truncated(value.to_string()));
    }
}

/// span 的 field 值。
///
/// tracing **不會**幫我們保留 span 的 field —— 必須在 `on_new_span` 自己存進 span
/// extensions，`on_event` 才有辦法回頭讀。`request_id` / `method` / `path` 都掛在
/// `routes.rs` 那條 request span 上，少了這步 DB 裡的 log 就對不到任何請求。
struct SpanFields(Map<String, Value>);

pub struct DbLogLayer {
    tx: mpsc::Sender<LogEntry>,
}

impl DbLogLayer {
    pub fn new(tx: mpsc::Sender<LogEntry>) -> Self {
        Self { tx }
    }
}

impl<S> Layer<S> for DbLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else { return };

        let mut visitor = FieldVisitor::default();
        attrs.record(&mut visitor);

        if !visitor.fields.is_empty() {
            span.extensions_mut().insert(SpanFields(visitor.fields));
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let meta = event.metadata();

        // 只落地 WARN / ERROR（業務事件走 admin_audit_logs，不在此表）
        if *meta.level() > tracing::Level::WARN {
            return;
        }

        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        let mut fields = visitor.fields;

        // 由內而外併入所屬 span 的 field；同名時內層優先（已存在就不覆蓋）
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope {
                if let Some(SpanFields(span_fields)) = span.extensions().get::<SpanFields>() {
                    for (key, value) in span_fields {
                        if !fields.contains_key(key) {
                            fields.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }

        // request_id 升成獨立欄位（有索引，是查線上問題的主要入口）。
        // `routes.rs` 的 span 在取不到 RequestId extension 時填 "-"，那視同沒有。
        let request_id = match fields.remove("request_id") {
            Some(Value::String(id)) if id != "-" => Some(id),
            _ => None,
        };

        let entry = LogEntry {
            level: meta.level().to_string(),
            message: visitor.message,
            target: meta.target().to_owned(),
            file: meta.file().map(str::to_owned),
            line: meta.line(),
            request_id,
            fields: (!fields.is_empty()).then_some(Value::Object(fields)),
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
    let request_ids: Vec<Option<&str>> = buf.iter().map(|e| e.request_id.as_deref()).collect();
    // 以 text[] 傳、SQL 端再 cast 成 jsonb：jsonb[] 的參數綁定沒必要在這裡冒險
    let fields: Vec<Option<String>> = buf
        .iter()
        .map(|e| e.fields.as_ref().map(|f| f.to_string()))
        .collect();

    let _ = sqlx::query(
        "INSERT INTO logs (level, message, target, file, line, request_id, fields)
         SELECT level, message, target, file, line, request_id, fields::jsonb
         FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::int[], $6::text[], $7::text[])
              AS t(level, message, target, file, line, request_id, fields)",
    )
    .bind(&levels)
    .bind(&messages)
    .bind(&targets)
    .bind(&files)
    .bind(&lines)
    .bind(&request_ids)
    .bind(&fields)
    .execute(pool)
    .await;

    buf.clear();
}
