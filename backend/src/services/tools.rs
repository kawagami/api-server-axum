use crate::errors::{unprocessable, AppError, SystemError};
use crate::structs::tools::ConversionDirection;
use zhconv::{zhconv, Variant};

/// 轉換輸入的長度上限。全域 body 上限是 10MB（routes.rs 的 RequestBodyLimitLayer），
/// 對「貼一篇文章來轉」這個用途來說過寬 —— 一篇長文的中文約 50–100KB，256KB 綽綽有餘。
///
/// 實測 zhconv 0.4.1 的 CPU 成本並不高（本機 10MB ≈ 28ms、1MB ≈ 2.5ms），所以這個上限
/// 主要是擋記憶體：1 核 1G 的 VPS 上，10MB 的 body 加上同樣大的輸出與中間配置，
/// 幾個併發請求就很有感。CPU 那一面交給下面的 spawn_blocking。
const CONVERT_TEXT_MAX_BYTES: usize = 256 * 1024;

pub async fn convert_text(
    text: String,
    direction: ConversionDirection,
) -> Result<String, AppError> {
    if text.len() > CONVERT_TEXT_MAX_BYTES {
        return Err(unprocessable(format!(
            "text 長度上限為 {} KB",
            CONVERT_TEXT_MAX_BYTES / 1024
        )));
    }

    let variant = variant_of(direction);

    // zhconv 是同步 CPU 工作，直接跑會佔住 async worker（1 核機上就是全站一起等）。
    // 慣例同 services/auth.rs 的 bcrypt 與 services/images.rs 的 process_image。
    tokio::task::spawn_blocking(move || zhconv(&text, variant))
        .await
        .map_err(|e| SystemError::Internal(format!("簡繁轉換執行失敗: {e}")).into())
}

fn variant_of(direction: ConversionDirection) -> Variant {
    match direction {
        ConversionDirection::T2s => Variant::ZhCN,
        ConversionDirection::S2t => Variant::ZhHant,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::RequestError;

    #[tokio::test]
    async fn t2s_converts_to_simplified() {
        let out = convert_text("漢語處理".to_string(), ConversionDirection::T2s)
            .await
            .unwrap();
        assert_eq!(out, "汉语处理");
    }

    #[tokio::test]
    async fn s2t_converts_to_traditional() {
        let out = convert_text("汉语处理".to_string(), ConversionDirection::S2t)
            .await
            .unwrap();
        assert_eq!(out, "漢語處理");
    }

    #[tokio::test]
    async fn oversized_text_is_rejected() {
        // 位元組數超限即擋，不必等 zhconv 跑完。
        let text = "a".repeat(CONVERT_TEXT_MAX_BYTES + 1);
        let err = convert_text(text, ConversionDirection::T2s)
            .await
            .expect_err("超過上限應回 422");
        assert!(matches!(
            err,
            AppError::RequestError(RequestError::UnprocessableContent(_))
        ));
    }
}
