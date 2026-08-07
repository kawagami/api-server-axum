use axum::http::HeaderMap;
use std::net::IpAddr;

/// 取請求來源 IP —— 全站唯一的判斷。
///
/// **只有確定流量都經 Cloudflare（`TRUST_CF_HEADER=true`）才信任 `CF-Connecting-IP`**，
/// 否則那個 header 可偽造：偽造值會進到 rate limit 的 key（等於繞過限流）、每日不重複
/// 到訪統計、後台連線列表與 `user_joined` 廣播。生產靠 nginx 在 server 層無條件覆寫成
/// `$remote_addr` 擋著，但不能只有一層 —— 直連 origin 或本地開發時就沒有那層。
///
/// 這個判斷原本在 `middleware/rate_limit.rs` 與 `routes/ws.rs` 各寫一份（逐字相同）。
/// 兩份的風險不是重複而是**漂移**：只改一邊的話，會出現「限流認得出真 IP、log 認不出」
/// 這種最難查的不一致。
pub fn client_ip(trust_cf_header: bool, headers: &HeaderMap, socket_ip: Option<IpAddr>) -> String {
    let socket_ip = socket_ip.map(|ip| ip.to_string());
    if trust_cf_header {
        headers
            .get("CF-Connecting-IP")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
            .or(socket_ip)
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        socket_ip.unwrap_or_else(|| "unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers_with_cf(ip: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("CF-Connecting-IP", ip.parse().unwrap());
        h
    }

    fn socket() -> Option<IpAddr> {
        Some("10.0.0.1".parse().unwrap())
    }

    /// 不信任 CF header 時，偽造的 header 必須完全無效 —— 這是防繞過限流的那道
    #[test]
    fn forged_cf_header_is_ignored_when_untrusted() {
        assert_eq!(client_ip(false, &headers_with_cf("1.2.3.4"), socket()), "10.0.0.1");
    }

    #[test]
    fn cf_header_wins_when_trusted() {
        assert_eq!(client_ip(true, &headers_with_cf("1.2.3.4"), socket()), "1.2.3.4");
    }

    /// 信任 CF 但這個請求沒帶 header（直連 origin）→ 退回 socket，不是 unknown
    #[test]
    fn falls_back_to_socket_then_unknown() {
        assert_eq!(client_ip(true, &HeaderMap::new(), socket()), "10.0.0.1");
        assert_eq!(client_ip(true, &HeaderMap::new(), None), "unknown");
        assert_eq!(client_ip(false, &HeaderMap::new(), None), "unknown");
    }
}
