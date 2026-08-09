use serde_json::{Map, Value};
use sqlx::{Pool, Postgres};
use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
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

const LEVEL_ERROR: u8 = 0;
const LEVEL_WARN: u8 = 1;
const LEVEL_INFO: u8 = 2;

/// `logs` 表的落地門檻，由 app_settings 的 `log_db_level` 熱更新。
///
/// **刻意是 process 全域的 static**：tracing subscriber 本來就是 process 全域，而
/// `DbLogLayer` 在 `AppState` 存在之前就得初始化（tracing 必須先於 DB pool）。
/// 為了它把一個 `Arc<AtomicU8>` 穿過 `AppStateInner::new` / `Settings::new` /
/// `routes::app` 四層建構子，換不到任何東西。
///
/// 預設 WARN = 設定載入前（啟動那幾百毫秒）與 DB 沒有該列時的行為，與加這個旋鈕
/// 之前完全一致。
static DB_LEVEL: AtomicU8 = AtomicU8::new(LEVEL_WARN);

/// 佇列滿而丟棄的筆數。這裡**不能**用 `tracing` 回報 —— 那會從 `on_event` 再遞迴回
/// `on_event`。改由 `log_writer` 的 tick 匯總後 `eprintln!` 進 stdout（docker logs），
/// 也就是這條管線壞掉時唯一還活著的通道。
static DROPPED: AtomicU64 = AtomicU64::new(0);

/// access log 的 target（`routes.rs` 的 `TraceLayer::on_response` 用）。
///
/// 不放在 crate 名底下是為了能單獨開關：它的量是「每個請求一行」，跟其他 INFO 差一個
/// 數量級，臨時要靜音時 `RUST_LOG=api_server_axum=info,http_access=off` 就好。
/// ⚠ 代價是 `main.rs::default_log_filter()` **必須明確列出這個 target** —— EnvFilter
/// 沒有 directive 命中的 target 一律當關閉，漏了就靜默失效。
pub const ACCESS_TARGET: &str = "http_access";

/// `log_db_level` 接受的值。**上限刻意停在 INFO**：再往下開 DEBUG 的話
/// `errors.rs` 會把每筆 404 與過期 token 寫進 PG，一隻掃描器打一輪就灌爆 1 核 1G 那台
/// 的 `logs` 表（那兩類刻意留在 debug，其餘 4xx 已在 WARN）。要看 DEBUG 只能調
/// `RUST_LOG` 走 stdout。
pub const DB_LEVEL_VALUES: &[&str] = &["ERROR", "WARN", "INFO"];

fn level_code(value: &str) -> Option<u8> {
    match value.to_ascii_uppercase().as_str() {
        "ERROR" => Some(LEVEL_ERROR),
        "WARN" => Some(LEVEL_WARN),
        "INFO" => Some(LEVEL_INFO),
        _ => None,
    }
}

fn code_level(code: u8) -> tracing::Level {
    match code {
        LEVEL_ERROR => tracing::Level::ERROR,
        LEVEL_INFO => tracing::Level::INFO,
        _ => tracing::Level::WARN,
    }
}

/// 套用 `log_db_level` 設定值。`Settings::reload` 每次重載都會呼叫。
///
/// ⚠️ 這個門檻只能在 `RUST_LOG` 這個**天花板底下**調 —— `EnvFilter` 掛在 registry 上
/// 是全域 filter，被它擋掉的 event 根本到不了這一層。生產的 release 預設是
/// `info,tower_http=warn`（見 `main.rs`），所以 INFO 剛好是可用的上限。
pub fn set_db_level(value: &str) {
    match level_code(value) {
        Some(code) => DB_LEVEL.store(code, Ordering::Relaxed),
        // PATCH 有驗證，走到這裡代表 DB 那列被手動改壞了。維持現值不動。
        None => tracing::error!("log_db_level 設定值 {value:?} 無法解析，維持原設定"),
    }
}

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

        // 落地門檻預設 WARN，可由 app_settings 的 log_db_level 熱調到 ERROR / INFO
        // （業務事件走 admin_audit_logs，不在此表）
        if *meta.level() > code_level(DB_LEVEL.load(Ordering::Relaxed)) {
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

        // 滿了就丟（寫入器追不上尖峰）。丟棄本身要看得見，否則「log 表突然變安靜」
        // 與「真的沒事發生」長得一模一樣 —— 計數交給 log_writer 匯總印出。
        if self.tx.try_send(entry).is_err() {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
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
                report_dropped();
            }
        }
    }
}

/// 把累積的丟棄數印進 stdout 並歸零。最多每 `FLUSH_INTERVAL_MS` 一行，不會自己變成洪水。
fn report_dropped() {
    let n = DROPPED.swap(0, Ordering::Relaxed);
    if n > 0 {
        eprintln!("log_writer: 佇列已滿，丟棄 {n} 筆 log（未落地 logs 表）");
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

    let result = sqlx::query(
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

    // 同理不能用 tracing 回報（會遞迴回 on_event，而且這條路正是壞掉的那條）。
    // 吞掉的話 `logs` 表停止寫入時零徵兆 —— 查線上問題的主入口靜默失效是最糟的失敗模式。
    if let Err(e) = result {
        eprintln!("log_writer: 落地 {} 筆 log 失敗: {e}", buf.len());
    }

    buf.clear();
}
