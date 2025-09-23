use crate::common::*;
use crate::dto::log_index_result::*;

#[async_trait]
pub trait NotificationService {
    async fn send_index_alert_message(
        &self,
        log_index_results: &[LogIndexResult],
    ) -> Result<(), anyhow::Error>;
}
