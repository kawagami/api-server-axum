/// query string 進 log 前的遮罩 —— 全站唯一的判斷。
///
/// 把 query 放進 request span 是為了讓「分頁 / 篩選類的 bug」能從 log 重現參數，
/// 但有幾個端點的 query 帶的是**憑證**：
/// - `/ws?ticket=` —— WS 握手票（30 秒 TTL、GETDEL 一次性，但仍是憑證）
/// - `/oauth/{provider}/callback?code=` —— OAuth authorization code
/// - torrent 的簽名下載連結（`sig=` / `signature=`）
///
/// CLAUDE.md 之所以規定「JWT 不走 WS URL query」，理由正是「會進 access log」——
/// 現在 access log 是我們自己在記的，那條規定就得在這裡兌現。
const SENSITIVE_KEYS: &[&str] = &[
    "ticket",
    "token",
    "code",
    "password",
    "secret",
    "sig",
    "signature",
    "key",
    "api_key",
];

/// 遮罩 query 中的敏感值，其餘原樣保留（`page=2&q=abc` 這種要看得到才有意義）。
pub fn redact_query(query: &str) -> String {
    query
        .split('&')
        .map(|pair| match pair.split_once('=') {
            Some((k, _)) if SENSITIVE_KEYS.contains(&k.to_ascii_lowercase().as_str()) => {
                format!("{k}=***")
            }
            _ => pair.to_string(),
        })
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_ordinary_params() {
        assert_eq!(redact_query("page=2&per_page=50&q=abc"), "page=2&per_page=50&q=abc");
    }

    /// 這幾個漏出去就是憑證外洩，是這個函式存在的唯一理由
    #[test]
    fn masks_credentials() {
        assert_eq!(redact_query("ticket=abc123"), "ticket=***");
        assert_eq!(redact_query("code=oauth-code&state=xyz"), "code=***&state=xyz");
        assert_eq!(redact_query("id=7&sig=deadbeef"), "id=7&sig=***");
    }

    /// key 名大小寫不該成為繞過的方式
    #[test]
    fn key_match_is_case_insensitive() {
        assert_eq!(redact_query("Token=abc"), "Token=***");
    }

    #[test]
    fn tolerates_malformed_pairs() {
        assert_eq!(redact_query(""), "");
        assert_eq!(redact_query("flag"), "flag");
        assert_eq!(redact_query("a=1&&b=2"), "a=1&&b=2");
    }
}
