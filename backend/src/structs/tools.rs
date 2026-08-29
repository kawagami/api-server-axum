use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConversionDirection {
    T2s,
    S2t,
}

#[derive(Deserialize)]
pub struct ConvertTextRequest {
    pub text: String,
    pub direction: ConversionDirection,
}

/// 只回轉換結果。**不要把 `original_text` 加回來** —— 原文是呼叫端自己傳進來的，
/// 回傳等於把上限 256KB 的 body 原封再送一遍（response 體積翻倍），
/// 而前端從來沒有讀它（唯一消費者是 `frontend/api/tools.ts` 的型別宣告）。
#[derive(Serialize)]
pub struct ConvertTextResponse {
    pub converted_text: String,
}
