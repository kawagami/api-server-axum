use crate::state::AppState;

pub enum AppJob {
    CleanupExpiredTorrents,
    CleanupUnusedImages,
    FetchStockDayAll,
    FetchBuybackPeriods,
    FetchNotes,
    FetchHistoricalClosingPrices,
    ConsumePendingStockChange,
    SyncBuybackToPending,
    CheckInvoiceLottery,
    CheckLottoWins,
}

impl AppJob {
    pub fn name(&self) -> &'static str {
        match self {
            AppJob::CleanupExpiredTorrents => "CleanupExpiredTorrents",
            AppJob::CleanupUnusedImages => "CleanupUnusedImages",
            AppJob::FetchStockDayAll => "FetchStockDayAll",
            AppJob::FetchBuybackPeriods => "FetchBuybackPeriods",
            AppJob::FetchNotes => "FetchNotes",
            AppJob::FetchHistoricalClosingPrices => "FetchHistoricalClosingPrices",
            AppJob::ConsumePendingStockChange => "ConsumePendingStockChange",
            AppJob::SyncBuybackToPending => "SyncBuybackToPending",
            AppJob::CheckInvoiceLottery => "CheckInvoiceLottery",
            AppJob::CheckLottoWins => "CheckLottoWins",
        }
    }

    pub fn cron_expression(&self) -> &str {
        match self {
            AppJob::CleanupExpiredTorrents => "0 30 * * * *",
            AppJob::CleanupUnusedImages => "0 0 * * * *",
            AppJob::FetchStockDayAll => "0 0 20 * * *",
            AppJob::FetchBuybackPeriods => "0 0 20 * * *",
            AppJob::FetchNotes => "0 0 19 * * *",
            AppJob::FetchHistoricalClosingPrices => "0 * * * * *",
            AppJob::ConsumePendingStockChange => "0 * * * * *",
            AppJob::SyncBuybackToPending => "0 10 20 * * *",
            // 每日 UTC 17:00（= UTC+8 隔日 01:00）；偵測新開獎期別才實際動作
            AppJob::CheckInvoiceLottery => "0 0 17 * * *",
            // 每日 UTC 13:30（= UTC+8 21:30）；已過大樂透/威力彩開獎時間，每天跑覆蓋四個開獎日
            AppJob::CheckLottoWins => "0 30 13 * * *",
        }
    }

    pub async fn run(&self, state: AppState) {
        match self {
            AppJob::CleanupExpiredTorrents => crate::jobs::cleanup_expired_torrents::run(state).await,
            AppJob::CleanupUnusedImages => crate::jobs::cleanup_unused_images::run(state).await,
            AppJob::FetchStockDayAll => crate::jobs::fetch_stock_day_all::run(state).await,
            AppJob::FetchBuybackPeriods => crate::jobs::fetch_buyback_periods::run(state).await,
            AppJob::FetchNotes => crate::jobs::fetch_notes::run(state).await,
            AppJob::FetchHistoricalClosingPrices => crate::jobs::fetch_historical_closing_prices::run(state).await,
            AppJob::ConsumePendingStockChange => crate::jobs::consume_pending_stock_change::run(state).await,
            AppJob::SyncBuybackToPending => crate::jobs::sync_buyback_to_pending::run(state).await,
            AppJob::CheckInvoiceLottery => crate::jobs::check_invoice_lottery::run(state).await,
            AppJob::CheckLottoWins => crate::jobs::check_lotto_wins::run(state).await,
        }
    }
}
