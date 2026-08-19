use crate::errors::AppError;
use crate::extract::Json;
use crate::middleware::rate_limit;
use crate::services::roster::{build_roster, resolve_plan};
use crate::state::AppState;
use crate::structs::roster::{RosterRequest, RosterResponse};
use axum::{middleware, routing::post, Router};

pub fn new(state: AppState) -> Router<AppState> {
    // 用 post 考量參數資料量可能很大。
    // 沿用 tools 的 bucket（20 req/60s）：同屬公開未認證的計算工具，語意一致，
    // 也不必為此再多一組常數。限流擋的是量,單發成本由 RosterRequest::validate 擋。
    Router::new()
        .route("/", post(calculate_roster))
        .layer(middleware::from_fn_with_state(
            state,
            rate_limit::tools_rate_limit,
        ))
}

/// 排班演算法本身在 `services::roster`（純函式、附測試）。這裡只做驗證與組回應。
pub async fn calculate_roster(
    Json(payload): Json<RosterRequest>,
) -> Result<Json<RosterResponse>, AppError> {
    // 一定要在進配置迴圈之前擋：這支端點無認證，days 未設上限時單一請求即可 OOM 整機
    payload
        .validate()
        .map_err(crate::errors::RequestError::UnprocessableContent)?;

    let plan = resolve_plan(&payload);
    let (data, warnings) = build_roster(&payload.names, payload.days, plan);

    Ok(Json(RosterResponse {
        status: "success".to_string(),
        data,
        plan,
        warnings,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::roster::RosterRule;

    fn payload(people: usize, days: u32) -> RosterRequest {
        RosterRequest {
            names: (0..people).map(|i| format!("p{i}")).collect(),
            days,
            rule: RosterRule::Fairness,
            morning_slots: None,
            night_slots: None,
            max_consecutive: None,
        }
    }

    /// 回應形狀是前端契約（`frontend/api/tools.ts` 的 `RosterResponse`）
    #[tokio::test]
    async fn returns_expected_json_shape() {
        let Json(response) = calculate_roster(Json(payload(6, 3))).await.unwrap();
        let value = serde_json::to_value(response).unwrap();
        assert_eq!(value["status"], "success");
        assert_eq!(value["data"][0]["id"], 1);
        assert_eq!(value["data"][0]["name"], "p0");
        assert_eq!(value["data"][0]["shifts"].as_array().unwrap().len(), 3);
        assert_eq!(value["plan"]["morning_slots"], 2);
        assert_eq!(value["plan"]["night_slots"], 2);
        assert_eq!(value["plan"]["rest_slots"], 2);
        assert_eq!(value["plan"]["max_consecutive"], 5);
        assert_eq!(value["warnings"].as_array().unwrap().len(), 0);
    }

    /// 警告碼的字面是契約：前端 `tools/roster/page.tsx` 的 `WARNING_KEYS` 靠它查 i18n
    #[tokio::test]
    async fn warning_codes_are_stable_snake_case() {
        let Json(response) = calculate_roster(Json(payload(2, 7))).await.unwrap();
        let value = serde_json::to_value(response).unwrap();
        let codes: Vec<&str> = value["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect();
        assert!(codes.contains(&"understaffed"), "{codes:?}");
        assert!(codes.contains(&"night_to_morning"), "{codes:?}");
        assert!(codes.contains(&"max_consecutive_exceeded"), "{codes:?}");
    }

    #[tokio::test]
    async fn invalid_payload_is_rejected() {
        assert!(calculate_roster(Json(payload(1, 0))).await.is_err());
        assert!(calculate_roster(Json(payload(0, 7))).await.is_err());
    }
}
