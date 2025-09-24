use crate::common::*;

// use crate::model::{
//     elastic_server_config::*, error_alarm_info::*, error_alarm_info_format::*,
//     receiver_email_config::*, system_config::*, total_config::*,
// };

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
    pub fn new() -> Self {
        let receiver_email_list: ReceiverEmailConfig =
            read_toml_from_file::<ReceiverEmailConfig>(&EMAIL_RECEIVER_PATH)
                .unwrap_or_else(|e| {
                    let err_msg: &str = "[ERROR][NotificationServicePub->new] Failed to retrieve information 'receiver_email_list'.";
                    error!("{} : {:?}", err_msg, e);
                    panic!("{} : {:?}", err_msg, e)
                });

        NotificationServiceImpl {
            receiver_email_list,
        }
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

    // #[doc = "색인 실패별 로그들을 완전실패/부분실패로 나눠주는 함수"]
    // /// # Arguments
    // /// * `error_alaram_infos` - 실패한 색인 정보
    // ///
    // /// # Returns
    // /// * Result<(), anyhow::Error>
    // fn get_error_clasification(
    //     &self,
    //     error_alaram_info: &ErrorAlarmInfo,
    //     err_alram_map: &mut HashMap<String, Vec<String>>,
    // ) -> Result<(), anyhow::Error> {
    //     let mut send_msg: String = String::from("");
    //     send_msg.push_str(&format!(
    //         " index name: {}\n",
    //         error_alaram_info.index_name()
    //     ));

    //     send_msg.push_str(&format!(
    //         "   - indexing type: {}\n",
    //         error_alaram_info.index_type()
    //     ));

    //     let key_name: String = if error_alaram_info.error_type() == "Full Error" {
    //         String::from("Full Error")
    //     } else {
    //         send_msg.push_str(&format!(
    //             "   - index cnt (declare cnt): {} ({})\n",
    //             error_alaram_info
    //                 .indexing_cnt_num
    //                 .to_formatted_string(&Locale::en),
    //             error_alaram_info
    //                 .declare_index_size
    //                 .to_formatted_string(&Locale::en)
    //         ));
    //         String::from("Partial Error")
    //     };

    //     err_alram_map
    //         .entry(key_name.clone())
    //         .or_default()
    //         .push(send_msg);

    //     Ok(())
    // }

    // #[doc = "색인이 실패했을 때, Telegram bot 을 통해서 알람을 보내주는 함수"]
    // /// # Arguments
    // /// * `error_alaram_infos` - 실패한 색인 정보들
    // ///
    // /// # Returns
    // /// * Result<(), anyhow::Error>
    // async fn send_indexing_failed_msg(
    //     &self,
    //     error_alaram_infos: &[ErrorAlarmInfoFormat],
    // ) -> Result<(), anyhow::Error> {
    //     let tele_repo: Arc<TelebotRepositoryImpl> = get_telegram_repo();

    //     let system_config: &'static SystemConfig= get_system_config_info();
    //     let msg_chunk_size: usize = *system_config.message_chunk_size();

    //     let mut err_alram_map: HashMap<String, Vec<String>> = HashMap::new();

    //     for chunk in error_alaram_infos.chunks(msg_chunk_size) {
    //         for item in chunk {
    //             self.get_error_clasification(item.error_alarm_info(), &mut err_alram_map)?;
    //         }

    //         let mut msg_format: String = String::from("[Elasticsearch Indexing Error!]\n");

    //         for (key, value) in err_alram_map {
    //             let error_type: String = key;
    //             let error_map: Vec<String> = value;

    //             msg_format.push_str(format!("[{}]\n", error_type).as_str());

    //             for err_msg in error_map {
    //                 msg_format.push_str(format!("{}\n", err_msg).as_str());
    //             }
    //         }

    //         /* Send Message */
    //         tele_repo.bot_send(&msg_format).await?;

    //         err_alram_map = HashMap::new(); /* Clear HashMap */
    //     }

    //     Ok(())
    // }

    // #[doc = "색인이 실패했을 때, mail 을 통해서 알람을 보내주는 함수"]
    // /// # Arguments
    // /// * `error_alaram_infos` - 실패한 색인 정보들
    // ///
    // /// # Returns
    // /// * Result<(), anyhow::Error>
    // async fn send_mail_to_receivers(
    //     &self,
    //     error_alarm_infos: &[ErrorAlarmInfoFormat],
    // ) -> Result<(), anyhow::Error> {
    //     let elastic_config: &'static ElasticServerConfig = get_mon_elastic_config_info();

    //     /* receiver email list */
    //     let receiver_email_list: &Vec<ReceiverEmail> = &self.receiver_email_list().emails;

    //     let email_subject: String = String::from("[Elasticsearch] Indexing ERROR Alarm");
    //     let mut inner_template: String = String::from("");
    //     let html_template: String = fs::read_to_string(Path::new(HTML_TEMPLATE_PATH.as_str()))?;

    //     for err_info in error_alarm_infos {
    //         let err_info_tag: String = err_info.error_alarm_info().convert_email_struct()?;
    //         inner_template.push_str(&err_info_tag);
    //     }

    //     let html_content: String = html_template
    //         .replace("{cluster_name}", elastic_config.elastic_cluster_name())
    //         .replace("{index_list}", &inner_template);

    //     let sql_conn: Arc<SqlServerRepositoryImpl> = get_sqlserver_repo();

    //     for receiver in receiver_email_list {
    //         match sql_conn
    //             .execute_imailer_procedure(receiver.email_id(), &email_subject, &html_content)
    //             .await
    //         {
    //             Ok(_) => {
    //                 info!("Successfully sent mail to {}", receiver.email_id());
    //             }
    //             Err(e) => {
    //                 error!("[ERROR][NotificationServicePub->send_mail_to_receivers] Failed sent mail to {} : {:?}", receiver.email_id(), e);
    //             }
    //         }
    //     }

    //     Ok(())
    // }
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
                    "📌📌📌📌📌 {} 📌📌📌📌📌\n",
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
            .generate_index_alert_html(log_index_results, elastic_config)
            .await?;

        //SMTP 버전
        self.send_message_to_receivers_smtp(&email_subject, &html_content).await?;
        
        // SP 버전
        // let sql_conn: Arc<SqlServerRepositoryImpl> = get_sqlserver_repo();

        // for receiver in receiver_email_list {
        //     match sql_conn
        //         .execute_imailer_procedure(receiver.email_id(), &email_subject, &html_content)
        //         .await
        //     {
        //         Ok(_) => {
        //             info!("Successfully sent index alert mail to {}", receiver.email_id());
        //         }
        //         Err(e) => {
        //             error!("[ERROR][NotificationServiceImpl->send_email_index_alert] Failed to send mail to {} : {:?}", receiver.email_id(), e);
        //         }
        //     }
        // }

        Ok(())
    }

    #[doc = ""]
    async fn generate_index_alert_html(
        &self,
        log_index_results: &[LogIndexResult],
        elastic_config: &ElasticServerConfig,
    ) -> anyhow::Result<String> {
        /* HTML 템플릿 파일 읽기 */
        let template_content: String = fs::read_to_string(&*HTML_TEMPLATE_PATH)?;

        /* 알람 행들 생성 */
        let alert_rows: String = self.generate_alert_rows(log_index_results).await?;

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
    async fn generate_alert_rows(
        &self,
        log_index_results: &[LogIndexResult],
    ) -> anyhow::Result<String> {
        let mut rows: String = String::new();

        println!("count: {:?}", log_index_results.len());

        for log_result in log_index_results {
            
            if let Some(alert_formats) = log_result.alert_index_format() {
                
            }   

            // rows.push_str(r#"<tr>"#);
            // rows.push_str(&format!(r#"
            //     <td>
            //         {}
            //     </td>
            // "#, log_result.index_name()));
            // rows.push_str(r#"</tr>"#);

            // rows.push_str(r#"
            //     <tr>
            //         <td></td>
            //     </tr>
            // "#);

            // if let Some(alert_formats) = log_result.alert_index_format() {
            //     for alert_format in alert_formats {
            //         let alerts_opt: &Option<Vec<AlertIndex>> = log_result.alert_index_format();
            //         let alerts_slice: &[AlertIndex] = alerts_opt.as_deref().unwrap_or(&[]);

            //         let history_table: String = self
            //             .generate_history_table_html(alert_format.index_name(), alerts_slice)
            //             .await?;

            //         rows.push_str(&format!(
            //             r#"<tr>
            //                 <td class="index-name">{}</td>
            //                 <td class="count-change">{}</td>
            //                 <td class="timestamp">{}</td>
            //                 <td>{}</td>
            //             </tr>"#,
            //             alert_format.index_name(),
            //             alert_format.cnt(),
            //             alert_format.timestamp(),
            //             history_table
            //         ));
            //     }
            // }
        }

        Ok(rows)
    }

    #[doc = ""]
    async fn generate_history_table_html(
        &self,
        index_name: &str,
        alert_index: &[AlertIndex],
    ) -> anyhow::Result<String> {
        let mut history_html: String = String::from(
            r#"
        <table class="history-table">
            <tr>
                <th>Time</th>
                <th>Count</th>
                <th>Change</th>
            </tr>
        "#,
        );

        // 예시 히스토리 데이터 (실제로는 DB에서 조회해야 함)
        // let sample_history = [
        //     ("2025-01-20 10:00:00", 100000, 0),
        //     ("2025-01-20 10:05:00", 105000, 5000),
        //     ("2025-01-20 10:10:00", 103000, -2000),
        //     ("2025-01-20 10:15:00", 120000, 17000),
        // ];

        // for (i, (time, count, change)) in alert_index.iter().enumerate() {
        //     let change_class = if *change > 0 {
        //         "count-increase"
        //     } else if *change < 0 {
        //         "count-decrease"
        //     } else {
        //         ""
        //     };

        //     let change_text = if *change > 0 {
        //         format!("+{}", change)
        //     } else if *change < 0 {
        //         format!("{}", change)
        //     } else {
        //         "0".to_string()
        //     };

        //     history_html.push_str(&format!(
        //         "<tr><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>",
        //         time, count, change_class, change_text
        //     ));
        // }

        for elem in alert_index {
            history_html.push_str(&format!(
                "<tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>",
                elem.timestamp(), 
                elem.cnt(), 
                elem.index_name()
            ));
        }

        history_html.push_str("</table>");

        Ok(history_html)
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
