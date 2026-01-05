use crate::errors::AppError;
use crate::state::AppStateV2;
use crate::structs::roster::{RosterRequest, RosterResponse, StaffShift}; // 引用上面的結構
use axum::{routing::post, Json, Router};
use rand::seq::SliceRandom; // 用於模擬隨機排班

pub fn new() -> Router<AppStateV2> {
    // 建議改用 post，因為 names 陣列可能很大
    Router::new().route("/", post(calculate_roster))
}

pub async fn calculate_roster(
    Json(payload): Json<RosterRequest>,
) -> Result<Json<RosterResponse>, AppError> {
    let mut rng = rand::thread_rng();
    let shift_options = vec!["早班", "晚班", "休"];
    let mut roster_result = Vec::new();

    tracing::info!("{}", payload.rule);

    // 模擬針對每個人生成排班
    for (index, name) in payload.names.iter().enumerate() {
        let mut shifts = Vec::new();

        // 根據 payload.days (例如 31) 生成假資料
        for _ in 0..payload.days {
            let random_shift = shift_options.choose(&mut rng).unwrap_or(&"休");
            shifts.push(random_shift.to_string());
        }

        roster_result.push(StaffShift {
            id: index + 1,
            name: name.clone(),
            shifts,
        });
    }

    Ok(Json(RosterResponse {
        status: "success".to_string(),
        data: roster_result,
    }))
}
