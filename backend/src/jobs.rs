pub mod aggregate_visitors;
pub mod check_invoice_lottery;
pub mod check_lotto_wins;
pub mod cleanup_expired_torrents;
pub mod cleanup_observability;
pub mod cleanup_unused_images;
pub mod collect_system_metrics;
pub mod consume_pending_stock_change;
pub mod fetch_buyback_periods;
pub mod fetch_gov_tenders;
pub mod fetch_historical_closing_prices;
pub mod fetch_stock_day_all;
pub mod sync_buyback_to_pending;

pub(super) async fn run_with_retries<F, Fut, E>(
    label: &str,
    max_attempts: u32,
    retry_delay: std::time::Duration,
    mut make_future: F,
) where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<(), E>>,
    E: std::fmt::Display,
{
    for attempt in 1..=max_attempts {
        match make_future().await {
            Ok(_) => {
                tracing::info!("{} success", label);
                return;
            }
            Err(e) => {
                if attempt < max_attempts {
                    tracing::warn!(
                        "{} fail (attempt {}/{}): {}, retry in {:?}",
                        label,
                        attempt,
                        max_attempts,
                        e,
                        retry_delay
                    );
                    tokio::time::sleep(retry_delay).await;
                } else {
                    tracing::error!(
                        "{} fail (attempt {}/{}): {}",
                        label,
                        attempt,
                        max_attempts,
                        e
                    );
                }
            }
        }
    }
}
