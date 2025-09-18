use crate::common::*;

use crate::model::error_alarm_info_format::*;

#[async_trait]
pub trait SmtpService {
    async fn send_message_to_receiver_html(
        &self,
        email_id: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<String, anyhow::Error>;
    async fn send_message_to_receivers(
        &self,
        error_alarm_infos: &Vec<ErrorAlarmInfoFormat>,
    ) -> Result<(), anyhow::Error>;
}
