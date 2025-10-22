use crate::common::*;

use crate::dto::{log_index_result::*, alarm::alarm_image_info::*};

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_index_alert_message(
        &self,
        log_index_results: &[LogIndexResult],
    ) -> Result<(), anyhow::Error>;
    async fn send_report_information_by_email(
        &self,
        email_subject: &str,
        html_content: &str,
        alarm_image_infos: &[AlarmImageInfo]
    ) -> Result<(), anyhow::Error>;
}