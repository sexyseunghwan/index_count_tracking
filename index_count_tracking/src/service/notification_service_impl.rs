use crate::common::*;

// use crate::model::{
//     elastic_server_config::*, error_alarm_info::*, error_alarm_info_format::*,
//     receiver_email_config::*, system_config::*, total_config::*,
// };

use crate::model::configs::{
    elastic_server_config::*, receiver_email_config::*, system_config::*, total_config::*,
};

use crate::traits::repository_traits::sqlserver_repository::SqlServerRepository;
use crate::traits::repository_traits::telegram_repository::*;
use crate::traits::service_traits::notification_service::*;

use crate::repository::{sqlserver_repository_impl::*, telegram_repository_impl::*};

use crate::utils_modules::io_utils::*;

use crate::env_configuration::env_config::*;

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
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
    // #[doc = "지정된 수신자 모두에게 이메일을 보내주는 함수"]
    // /// # Arguments
    // /// * `error_alarm_infos` - Index error informations
    // ///
    // /// # Returns
    // /// * Result<(), anyhow::Error>
    // async fn send_message_to_receivers(
    //     &self,
    //     error_alarm_infos: &[ErrorAlarmInfoFormat],
    // ) -> Result<(), anyhow::Error> {
    //     /* Telegram 이나 Imailer 가 통신되지 않을 경우를 고려한다. */
    //     /* 1. Telegram 알람 전송 */
    //     let telegram = async {
    //         if let Err(e) = self.send_indexing_failed_msg(error_alarm_infos).await {
    //             error!(
    //                 "[ERROR][NotificationServicePub->send_message_to_receivers][telegram]{:?}",
    //                 e
    //             );
    //         }
    //     };

    //     /* 2. Imailer 알람 전송 */
    //     let mail = async {
    //         if let Err(e) = self.send_mail_to_receivers(error_alarm_infos).await {
    //             error!(
    //                 "[ERROR][NotificationServicePub->send_message_to_receivers][imailer]{:?}",
    //                 e
    //             );
    //         }
    //     };

    //     /* 병렬 실행 */
    //     let ((), ()) = tokio::join!(telegram, mail);

    //     Ok(())
    // }
}
