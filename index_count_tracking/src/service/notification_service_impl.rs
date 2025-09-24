use crate::common::*;

use crate::model::{
    configs::{
        elastic_server_config::*, receiver_email_config::*, system_config::*, total_config::*, smtp_config::*
    },
    index::alert_index::*,
};

use crate::traits::repository_traits::{sqlserver_repository::*, telegram_repository::*};
use crate::traits::service_traits::notification_service::*;

use crate::repository::{sqlserver_repository_impl::*, telegram_repository_impl::*};

use crate::utils_modules::io_utils::*;

use crate::env_configuration::env_config::*;

use crate::dto::log_index_result::*;

#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct NotificationServiceImpl {
    receiver_email_list: ReceiverEmailConfig,
}

impl NotificationServiceImpl {
    #[doc = "NotificationServicePub 구조체의 생성자"]
    pub fn new() -> anyhow::Result<Self> {
        let receiver_email_list = read_toml_from_file::<ReceiverEmailConfig>(&EMAIL_RECEIVER_PATH)
            .map_err(|e| {
                let err_msg = "[ERROR][NotificationServiceImpl->new] Failed to retrieve information 'receiver_email_list'.";
                error!("{} : {:?}", err_msg, e);
                anyhow!("{} : {:?}", err_msg, e)
            })?;

        Ok(NotificationServiceImpl {
            receiver_email_list,
        })
    }

    #[doc = "수신자에게 이메일을 보내주는 함수"]
    /// # Arguments
    /// * `email_id`        - 수신자 이메일 주소
    /// * `subject`         - 이메일 제목
    /// * `html_content`    - 이메일 양식 (HTML 양식)
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_message_to_receiver_html(
        &self,
        smtp_config: &SmtpConfig,
        email_id: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<String, anyhow::Error> {

        let email: Message = Message::builder()
            .from(smtp_config.credential_id.parse()?)
            .to(email_id.parse()?)
            .subject(subject)
            .multipart(
                MultiPart::alternative().singlepart(SinglePart::html(html_content.to_string())),
            )?;

        let creds: Credentials = Credentials::new(
            smtp_config.credential_id().to_string(),
            smtp_config.credential_pw().to_string(),
        );

        let mailer: AsyncSmtpTransport<lettre::Tokio1Executor> =
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(
                smtp_config.smtp_name().as_str(),
            )?
            .credentials(creds)
            .build();

        match mailer.send(email).await {
            Ok(_) => Ok(email_id.to_string()),
            Err(e) => Err(anyhow!("{:?} : Failed to send email to {} ", e, email_id)),
        }
    }



    #[doc = ""]
    async fn send_message_to_receivers_smtp(&self, email_subject: &str, html_content: &str) -> anyhow::Result<()> {
        
        /* receiver email list */
        let receiver_email_list: &Vec<ReceiverEmail> = &self.receiver_email_list().emails;

        let smtp_config: &SmtpConfig = get_smtp_config_info();
        
        if smtp_config.async_process_yn {
            /* ASYNC TASK */
            let tasks = receiver_email_list.iter().map(|receiver| {
                let email_id: &String = receiver.email_id();
                self.send_message_to_receiver_html(smtp_config, email_id.as_str(), &email_subject, &html_content)
            });

            let results: Vec<Result<String, anyhow::Error>> = join_all(tasks).await;

            for result in results {
                match result {
                    Ok(succ_email_id) => info!("Email sent successfully: {}", succ_email_id),
                    Err(e) => error!(
                        "[Error][send_message_to_receivers()] Failed to send email: {:?}",
                        e
                    ),
                }
            }
        } else {
            /* Not Async */
            for receiver in receiver_email_list {
                let email_id: &String = receiver.email_id();
                self.send_message_to_receiver_html(
                    smtp_config,
                    email_id.as_str(),
                    "[Elasticsearch] Index removed list",
                    &html_content,
                )
                .await?;
            }
        }

        Ok(())
    }

    
    #[doc = "색인 카운트 알람을 텔레그램으로 전송하는 함수 (메시지 길이 제한을 고려한 chunk 방식)"]
    async fn send_telegram_index_alert(
        &self,
        log_index_results: &[LogIndexResult],
    ) -> anyhow::Result<()> {
        let tele_repo: Arc<TelebotRepositoryImpl> = get_telegram_repo();
        let system_config: &'static SystemConfig = get_system_config_info();
        let msg_chunk_size: usize = *system_config.message_chunk_size();

        /* LogIndexResult를 chunk 단위로 처리 */
        for chunk in log_index_results.chunks(msg_chunk_size) {
            let mut msg_format: String = String::from("🚨 [Index Count Alert] 🚨\n\n");

            for log_result in chunk {
                msg_format.push_str(&format!(
                    "📌 {} 📌\n",
                    log_result.index_name()
                ));

                if let Some(alert_formats) = log_result.alert_index_format() {
                    for alert_format in alert_formats {
                        msg_format.push_str(&format!(
                            "📊 Index: {}\n💾 Count: {}\n🕐 Time: {}\n\n",
                            alert_format.index_name(),
                            alert_format.cnt(),
                            alert_format.timestamp()
                        ));
                    }
                }
            }

            msg_format.push_str("⚠️ Please check the index status immediately!");

            /* 각 chunk별로 메시지 전송 */
            tele_repo.bot_send(&msg_format).await?;
        }

        Ok(())
    }

    #[doc = ""]
    async fn send_email_index_alert(
        &self,
        log_index_results: &[LogIndexResult],
    ) -> anyhow::Result<()> {
        let elastic_config: &'static ElasticServerConfig = get_mon_elastic_config_info();
        let receiver_email_list: &Vec<ReceiverEmail> = &self.receiver_email_list().emails;

        let email_subject: String =
            String::from("[Elasticsearch] Index Document Count Change Detected");

        let html_content: String = self
            .generate_index_alert_html(log_index_results, elastic_config)?;

        /* SMTP 버전 -> 온라인망 사용용*/
        //self.send_message_to_receivers_smtp(&email_subject, &html_content).await?;
        
        /* SP 버전 */ 
        let sql_conn: Arc<SqlServerRepositoryImpl> = get_sqlserver_repo();

        for receiver in receiver_email_list {
            match sql_conn
                .execute_imailer_procedure(receiver.email_id(), &email_subject, &html_content)
                .await
            {
                Ok(_) => {
                    info!("Successfully sent index alert mail to {}", receiver.email_id());
                }
                Err(e) => {
                    error!("[ERROR][NotificationServiceImpl->send_email_index_alert] Failed to send mail to {} : {:?}", receiver.email_id(), e);
                }
            }
        }

        Ok(())
    }

    #[doc = ""]
    fn generate_index_alert_html(
        &self,
        log_index_results: &[LogIndexResult],
        elastic_config: &ElasticServerConfig,
    ) -> anyhow::Result<String> {
        /* HTML 템플릿 파일 읽기 */
        let template_content: String = fs::read_to_string(&*HTML_TEMPLATE_PATH)?;

        /* 알람 행들 생성 */
        let alert_rows: String = self.generate_alert_rows(log_index_results)?;

        /* 템플릿의 플레이스홀더 교체 */
        let html_content: String = template_content
            .replace("{cluster_name}", elastic_config.elastic_cluster_name())
            .replace(
                "{alert_time}",
                &chrono::Utc::now()
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
            )
            .replace("{alert_rows}", &alert_rows);

        Ok(html_content)
    }

    #[doc = ""]
    fn generate_alert_rows(
        &self,
        log_index_results: &[LogIndexResult],
    ) -> anyhow::Result<String> {
        let mut rows: String = String::new();

        for log_result in log_index_results {

            if let Some(alert_formats) = log_result.alert_index_format() {

                rows.push_str(&format!(
                    r#"<tr>
                        <td style="border: 1px solid #ddd; padding: 12px; text-align: left; vertical-align: top; background-color: #fff;">{}</td>
                        <td style="border: 1px solid #ddd; padding: 12px; text-align: left; vertical-align: top; background-color: #fff;">{}</td>
                        <td style="border: 1px solid #ddd; padding: 12px; text-align: left; vertical-align: top; background-color: #fff;">{}</td>
                        <td style="border: 1px solid #ddd; padding: 12px; text-align: left; vertical-align: top; background-color: #fff;">{}</td>
                
                    </tr>"#, 
                    log_result.index_name(),
                    log_result.cur_cnt(),
                    log_result.fluctuation_val(),
                    self.generate_history_table_html(alert_formats)
                ));
            }   
        }

        Ok(rows)
    }

    #[doc = ""]
    fn generate_history_table_html(
        &self,
        alert_indexes: &[AlertIndex],
    ) -> String {
        
        let mut inner_div: String = String::from(r#""#);

        for alert_index in alert_indexes {
            inner_div.push_str(&format!(r#"<div>{} -> {}</div>"#, alert_index.timestamp(), alert_index.cnt()));
        } 

        let history_divs: String = String::from(
            &format!(r#"
                <div style="color: #555; font-size: 14px; line-height: 1.5;">
                {}
                </div>
            "#, inner_div)
        );
        
        history_divs
    }
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
    #[doc = ""]
    async fn send_index_alert_message(
        &self,
        log_index_results: &[LogIndexResult],
    ) -> Result<(), anyhow::Error> {
        /* Telegram 이나 Imailer 가 통신되지 않을 경우를 고려한다. */
        /* 1. Telegram 알람 전송 */
        let telegram = async {
            if let Err(e) = self.send_telegram_index_alert(log_index_results).await {
                error!(
                    "[NotificationServiceImpl->send_index_alert_message][telegram] {:?}",
                    e
                );
            }
        };

        /* 2. Imailer 알람 전송 */
        let mail = async {
            if let Err(e) = self.send_email_index_alert(log_index_results).await {
                error!(
                    "[NotificationServiceImpl->send_index_alert_message][imailer] {:?}",
                    e
                );
            }
        };

        /* 병렬 실행 */
        let ((), ()) = tokio::join!(telegram, mail);

        Ok(())
    }
}
