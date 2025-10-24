use crate::common::*;

use crate::dto::{alarm::alarm_image_info::*, log_index_result::*};

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
        alarm_image_infos: &[AlarmImageInfo],
    ) -> Result<(), anyhow::Error>;
}
