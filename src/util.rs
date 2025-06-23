use std::cmp::min;
use std::time::Duration;

pub async fn sleep(mut duration: Duration, cancellation_token: &tokio_util::sync::CancellationToken) {
    const CHECK_INTERVAL: Duration = Duration::from_millis(100);
    loop {
        let wait_duration = min(CHECK_INTERVAL, duration);
        tokio::time::sleep(wait_duration).await;
        duration = duration.saturating_sub(CHECK_INTERVAL);
        
        if cancellation_token.is_cancelled() || duration.is_zero() {
            return;
        }
    }
}