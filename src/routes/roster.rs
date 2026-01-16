use crate::errors::AppError;
use crate::state::AppStateV2;
use crate::structs::roster::{RosterRequest, RosterResponse, StaffShift};
use axum::{routing::post, Json, Router};
use std::collections::VecDeque;

pub fn new() -> Router<AppStateV2> {
    // 用 post 考量參數資料量可能很大
    Router::new().route("/", post(calculate_roster))
}

pub async fn calculate_roster(
    Json(payload): Json<RosterRequest>,
) -> Result<Json<RosterResponse>, AppError> {
    let names = payload.names;
    let days = payload.days as usize;
    let rule = payload.rule;

    // 1. 定義基礎班別循環
    // 根據規則調整班別比例
    let base_pattern = match rule.as_str() {
        "morning_heavy" => vec!["早班", "早班", "晚班", "休"],
        "night_heavy" => vec!["晚班", "晚班", "早班", "休"],
        _ => vec!["早班", "晚班", "休"], // 預設平均分配 (fairness)
    };

    let mut roster_result = Vec::new();

    // 2. 為每位員工生成班表
    for (idx, name) in names.into_iter().enumerate() {
        let mut shifts = Vec::new();

        // 為了不讓所有人排到一樣的班，我們根據員工 index 給予不同的起始偏移
        // 例如：A 從「早」開始，B 從「晚」開始，C 從「休」開始
        let offset = idx % base_pattern.len();
        let mut pattern_queue: VecDeque<&str> = base_pattern.iter().copied().collect();
        pattern_queue.rotate_left(offset);

        // 3. 填充指定天數的班表
        for d in 0..days {
            // 取得當前循環中的班別
            let shift = pattern_queue[d % pattern_queue.len()];
            shifts.push(shift.to_string());
        }

        roster_result.push(StaffShift {
            id: idx + 1,
            name,
            shifts,
        });
    }

    // 4. 回傳結果
    Ok(Json(RosterResponse {
        status: "success".to_string(),
        data: roster_result,
    }))
}
