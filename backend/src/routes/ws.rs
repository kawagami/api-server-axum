use crate::extract::{Json, Query};
use crate::{
    errors::AppError,
    middleware::auth,
    services::{stats as stats_service, ws as ws_service},
    state::{AppState, DisplayTrackedConnection},
    structs::{auth::AuthenticatedUser, roles::Perm, ws::SendMessageRequest},
};
use axum::{
    extract::{connect_info::ConnectInfo, ws::WebSocketUpgrade, Extension, State},
    http::HeaderMap,
    middleware,
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Router
};
use axum_extra::{headers, TypedHeader};
use tracing::Instrument;
use std::net::SocketAddr;

// socket 生命週期（收訊迴圈、ping/pong、遊戲分派、斷線清理）在 `services::ws`；
// 這裡只管握手（身分、per-IP 上限、到訪採集、span）與三支 admin HTTP 端點。

#[derive(serde::Deserialize)]
struct WsQuery {
    ticket: Option<String>,
}

pub fn new(state: AppState) -> Router<AppState> {
    // 刻意直接掛 authorize_and_load 而非 super::with_auth（＝不進 audit）：/ticket 是
    // 每次 WS 重連都會打的高頻端點，寫進 admin_audit_logs 只會把稽核表灌滿噪音。
    // 其餘 admin 模組一律用 with_auth。
    let admin_routes = Router::new()
        .route("/connections", get(list_connections))
        .route("/messages", post(send_message))
        .route("/ticket", post(create_ws_ticket))
        .layer(middleware::from_fn_with_state(
            state,
            auth::authorize_and_load,
        ));

    Router::new()
        .route("/", any(ws_handler))
        .merge(admin_routes)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query): Query<WsQuery>,
    req_headers: HeaderMap,
) -> Response {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    // admin 身分改用一次性 ticket（POST /ws/ticket 換發，30 秒 TTL），
    // JWT 不再走 URL query，避免 token 進 access log
    let user_email = match query.ticket {
        Some(ticket) => ws_service::consume_ticket(&state, &ticket).await,
        None => None,
    };
    // 與 middleware/rate_limit.rs 同一條規則（同一個函式）：只有確定流量都經 Cloudflare
    // （TRUST_CF_HEADER=true）才信任這個 header
    let real_ip = crate::utils::net::client_ip(
        state.get_config().trust_cf_header,
        &req_headers,
        Some(addr.ip()),
    );
    // debug：每個訪客一行、且帶 IP / UA / email 個資。連線清單走 GET /ws/connections，
    // 新連線走 user_joined（只推 admin），這行只是本機開發時的方便，不該進生產 stdout。
    tracing::debug!("{real_ip} connected ({}) email={:?}", user_agent, user_email);

    // 匿名連線的 per-IP 上限。**握手前就擋**：被擋的連線不進 `connections`、不記到訪、
    // 不會在任何遊戲 hub 佔桌／佔房，也不會有 socket task 與 ping task。
    // admin（ticket 身分）不受限 —— 後台本來就會開好幾個分頁，且那條路徑已要求 `ws:read`。
    if user_email.is_none() {
        let existing = ws_service::ip_connection_count(&state, &real_ip).await;
        if existing >= ws_service::MAX_CONNECTIONS_PER_IP {
            // 與 middleware/rate_limit.rs 同理：限流觸發是安全訊號，必須留下紀錄，
            // 否則只有對方收到 429、我方零紀錄。
            tracing::warn!(
                "WS 連線數超限：ip={real_ip} 已有 {existing} 條，上限 {}，拒絕握手",
                ws_service::MAX_CONNECTIONS_PER_IP
            );
            return (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                "too many websocket connections",
            )
                .into_response();
        }
    }

    // 每日不重複到訪統計：以 WS 握手為採集點（天然濾掉不跑 JS 的 bot），
    // 去重元素 = ip|ua。best-effort，不阻塞連線。
    {
        let visit_state = state.clone();
        let ip = real_ip.clone();
        let ua = user_agent.clone();
        tokio::spawn(async move {
            stats_service::record_visit(&visit_state, &ip, &ua).await;
        });
    }

    // WS 的 log 全部掛在這條 span 底下。少了它，upgrade 之後的 socket task 既沒有
    // `request_id`（那是 task-local，留在握手那個 task 裡）也沒有任何結構化欄位 ——
    // `logs` 表裡只剩一段夾著 SocketAddr 的字串，對不回任何一條連線、也對不回握手請求。
    // 必須顯式 `instrument`：`on_upgrade` 的 future 由 hyper 在另一個 task 上驅動，
    // 當下的 span context 不會自己跟過去。
    let span = tracing::info_span!(
        "ws",
        conn = %addr,
        ip = %real_ip,
        email = %user_email.as_deref().unwrap_or("-"),
        // `DbLogLayer` 把 "-" 視同沒有，不會落地成假的 request_id
        request_id = %crate::middleware::request_id::current_request_id()
            .unwrap_or_else(|| "-".to_string()),
    );

    ws.on_upgrade(move |socket| {
        ws_service::handle_socket(socket, addr, state, user_email, real_ip, user_agent).instrument(span)
    })
    .into_response()
}

// 原有的獲取所有連接的端點
async fn list_connections(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DisplayTrackedConnection>>, AppError> {
    auth_user.require_permission(Perm::WsRead)?;

    Ok(Json(ws_service::list_connections(&state).await))
}

/// 失敗一律回非 2xx。舊版對「位址格式錯 / 連線不存在 / 送出失敗」都回 200 加一段錯誤字串，
/// 呼叫端只看 status 的話會把失敗顯示成成功。
async fn send_message(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(params): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth_user.require_permission(Perm::WsRead)?;

    ws_service::send_admin_message(&state, &auth_user.name, &params.addr, &params.message).await?;
    Ok(Json(serde_json::json!({ "sent": true })))
}

/// 換發 WS 一次性連線票（30 秒 TTL）。登入中的 admin 用它連 WS 取得管理員身分，
/// token 本體不再出現在 WS URL。
///
/// 權限門檻必須與 `list_connections` 一致（同為 `ws:read`）：ticket 換來的連線會被
/// 標成 admin 身分，因而收得到 `broadcast_to_admins` 推的 `user_joined` / `user_left`，
/// 那兩個事件的 payload 含 `real_ip` 與 `user_email`。少了這道檢查，沒有 ws:read 的管理員
/// HTTP 端查不到連線清單，卻能改走 WS 拿到同樣的個資。
async fn create_ws_ticket(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth_user.require_permission(Perm::WsRead)?;

    let ticket = ws_service::issue_ticket(&state, &auth_user.name).await?;
    Ok(Json(serde_json::json!({ "ticket": ticket })))
}
