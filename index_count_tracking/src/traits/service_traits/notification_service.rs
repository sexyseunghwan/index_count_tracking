use crate::common::*;
use crate::dto::log_index_result::*;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_index_alert_message(
        &self,
        log_index_results: &[LogIndexResult],
    ) -> Result<(), anyhow::Error>;

    async fn send_daily_report_email(
        &self,
        email_subject: &str,
        html_content: &str,
        chart_img_path_list: &[PathBuf],
    ) -> Result<(), anyhow::Error>;
}
