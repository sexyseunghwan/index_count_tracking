use crate::common::*;

use crate::model::{
    configs::{
        elastic_server_config::*, receiver_email_config::*, smtp_config::*, system_config::*,
        total_config::*,
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
    #[doc = r#"
        NotificationServiceImpl 구조체의 생성자 함수.

        1. 이메일 수신자 설정 파일(`EMAIL_RECEIVER_PATH`)을 읽어온다
        2. TOML 형식의 설정 파일을 `ReceiverEmailConfig` 구조체로 파싱
        3. 파싱에 실패할 경우 상세한 에러 메시지와 함께 실패 반환
        4. 성공 시 `NotificationServiceImpl` 인스턴스를 생성하여 반환

        이 생성자는 알람 서비스 초기화 시 필요한 이메일 수신자 목록을
        미리 로드하여 메모리에 캐시함으로써 알람 발송 시 성능을 최적화한다.

        # Returns
        * `Ok(NotificationServiceImpl)` - 초기화된 알람 서비스 인스턴스
        * `Err(anyhow::Error)` - 설정 파일 읽기 또는 파싱 실패 시

        # Errors
        * 설정 파일이 존재하지 않거나 읽기 권한이 없을 때
        * TOML 형식이 잘못되어 파싱에 실패할 때
        * `ReceiverEmailConfig` 구조체로 변환할 수 없을 때
    "#]
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

    #[doc = r#"
        개별 수신자에게 HTML 형식의 이메일을 발송하는 비동기 함수.

        1. 이메일 메시지 객체를 생성하고 발신자/수신자/제목/본문을 설정
        2. SMTP 서버 인증 정보를 바탕으로 Credentials 객체 생성
        3. `AsyncSmtpTransport`를 통해 SMTP 서버와 연결 설정
        4. 구성된 메일러를 통해 실제 이메일 발송 시도
        5. 발송 성공 시 수신자 이메일 주소 반환, 실패 시 에러 반환

        이 함수는 lettre 크레이트를 사용하여 비동기적으로 이메일을 발송하며,
        HTML 멀티파트 메시지를 지원한다.

        # Arguments
        * `smtp_config` - SMTP 서버 설정 정보 (서버명, 인증정보 포함)
        * `email_id` - 수신자 이메일 주소
        * `subject` - 이메일 제목
        * `html_content` - HTML 형식의 이메일 본문

        # Returns
        * `Ok(String)` - 발송 성공 시 수신자 이메일 주소
        * `Err(anyhow::Error)` - 이메일 구성 또는 발송 실패 시

        # Errors
        * 이메일 주소 파싱 실패
        * SMTP 서버 연결 실패
        * 인증 실패
        * 메시지 전송 실패
    "#]
    #[allow(dead_code)]
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
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(smtp_config.smtp_name().as_str())?
                .credentials(creds)
                .build();

        match mailer.send(email).await {
            Ok(_) => Ok(email_id.to_string()),
            Err(e) => Err(anyhow!("{:?} : Failed to send email to {} ", e, email_id)),
        }
    }

    #[doc = r#"
        개별 수신자에게 HTML 형식의 이메일과 차트 이미지 첨부파일을 발송하는 비동기 함수.

        1. 이메일 메시지 객체를 생성하고 발신자/수신자/제목/본문을 설정
        2. 첨부된 차트 이미지 파일들을 읽어서 이메일에 첨부
        3. SMTP 서버 인증 정보를 바탕으로 Credentials 객체 생성
        4. `AsyncSmtpTransport`를 통해 SMTP 서버와 연결 설정
        5. 구성된 메일러를 통해 실제 이메일 발송 시도
        6. 발송 성공 시 수신자 이메일 주소 반환, 실패 시 에러 반환

        # Arguments
        * `smtp_config` - SMTP 서버 설정 정보 (서버명, 인증정보 포함)
        * `email_id` - 수신자 이메일 주소
        * `subject` - 이메일 제목
        * `html_content` - HTML 형식의 이메일 본문
        * `attachments` - 첨부할 파일 경로 목록

        # Returns
        * `Ok(String)` - 발송 성공 시 수신자 이메일 주소
        * `Err(anyhow::Error)` - 이메일 구성 또는 발송 실패 시
    "#]
    #[allow(dead_code)]
    async fn send_message_to_receiver_with_attachments(
        &self,
        smtp_config: &SmtpConfig,
        email_id: &str,
        subject: &str,
        html_content: &str,
        attachments: &[std::path::PathBuf],
    ) -> Result<String, anyhow::Error> {
        use lettre::message::{Attachment, header};

        let mut multipart = MultiPart::mixed().multipart(
            MultiPart::alternative().singlepart(SinglePart::html(html_content.to_string())),
        );

        // Add attachments
        for attachment_path in attachments {
            let file_content = tokio::fs::read(attachment_path).await?;
            let filename = attachment_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("chart.png");

            let attachment = Attachment::new(filename.to_string())
                .body(file_content, header::ContentType::parse("image/png")?);

            multipart = multipart.singlepart(attachment);
        }

        let email: Message = Message::builder()
            .from(smtp_config.credential_id.parse()?)
            .to(email_id.parse()?)
            .subject(subject)
            .multipart(multipart)?;

        let creds: Credentials = Credentials::new(
            smtp_config.credential_id().to_string(),
            smtp_config.credential_pw().to_string(),
        );

        let mailer: AsyncSmtpTransport<lettre::Tokio1Executor> =
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(smtp_config.smtp_name().as_str())?
                .credentials(creds)
                .build();

        match mailer.send(email).await {
            Ok(_) => Ok(email_id.to_string()),
            Err(e) => Err(anyhow!("{:?} : Failed to send email to {} ", e, email_id)),
        }
    }

    #[doc = r#"
        SMTP 서버를 통해 수신자 목록에게 이메일을 일괄 발송하는 함수.

        1. 설정된 SMTP 정보와 수신자 목록을 가져온다
        2. `async_process_yn` 설정에 따라 처리 방식 결정:
           - true: 비동기 병렬 처리로 모든 이메일을 동시 발송 (성능 우선)
           - false: 순차적 동기 처리로 하나씩 발송 (안정성 우선)
        3. 각 수신자별로 `send_message_to_receiver_html` 호출하여 개별 이메일 발송
        4. 발송 결과를 로깅하되, 개별 실패가 전체 프로세스를 중단하지 않음

        # Arguments
        * `email_subject` - 이메일 제목
        * `html_content` - HTML 형식의 이메일 본문

        # Returns
        * `anyhow::Result<()>` - 전체 프로세스 성공/실패 여부
    "#]
    #[allow(dead_code)]
    async fn send_message_to_receivers_smtp(
        &self,
        email_subject: &str,
        html_content: &str,
    ) -> anyhow::Result<()> {
        /* receiver email list */
        let receiver_email_list: &Vec<ReceiverEmail> = &self.receiver_email_list().emails;

        let smtp_config: &SmtpConfig = get_smtp_config_info();

        if smtp_config.async_process_yn {
            /* ASYNC TASK */
            let tasks = receiver_email_list.iter().map(|receiver| {
                let email_id: &String = receiver.email_id();
                self.send_message_to_receiver_html(
                    smtp_config,
                    email_id.as_str(),
                    email_subject,
                    html_content,
                )
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
                    html_content,
                )
                .await?;
            }
        }

        Ok(())
    }

    #[doc = r#"
        SMTP 서버를 통해 수신자 목록에게 첨부파일이 포함된 이메일을 일괄 발송하는 함수.

        1. 설정된 SMTP 정보와 수신자 목록을 가져온다
        2. `async_process_yn` 설정에 따라 처리 방식 결정:
           - true: 비동기 병렬 처리로 모든 이메일을 동시 발송 (성능 우선)
           - false: 순차적 동기 처리로 하나씩 발송 (안정성 우선)
        3. 각 수신자별로 `send_message_to_receiver_with_attachments` 호출하여 개별 이메일 발송
        4. 발송 결과를 로깅하되, 개별 실패가 전체 프로세스를 중단하지 않음

        # Arguments
        * `email_subject` - 이메일 제목
        * `html_content` - HTML 형식의 이메일 본문
        * `attachments` - 첨부할 파일 경로 목록

        # Returns
        * `anyhow::Result<()>` - 전체 프로세스 성공/실패 여부
    "#]
    #[allow(dead_code)]
    async fn send_message_to_receivers_smtp_with_attachments(
        &self,
        email_subject: &str,
        html_content: &str,
        attachments: &[std::path::PathBuf],
    ) -> anyhow::Result<()> {
        /* receiver email list */
        let receiver_email_list: &Vec<ReceiverEmail> = &self.receiver_email_list().emails;

        let smtp_config: &SmtpConfig = get_smtp_config_info();

        if smtp_config.async_process_yn {
            /* ASYNC TASK */
            let tasks = receiver_email_list.iter().map(|receiver| {
                let email_id: &String = receiver.email_id();
                self.send_message_to_receiver_with_attachments(
                    smtp_config,
                    email_id.as_str(),
                    email_subject,
                    html_content,
                    attachments,
                )
            });

            let results: Vec<Result<String, anyhow::Error>> = join_all(tasks).await;

            for result in results {
                match result {
                    Ok(succ_email_id) => info!("Email sent successfully: {}", succ_email_id),
                    Err(e) => error!(
                        "[Error][send_message_to_receivers_with_attachments()] Failed to send email: {:?}",
                        e
                    ),
                }
            }
        } else {
            /* Not Async */
            for receiver in receiver_email_list {
                let email_id: &String = receiver.email_id();
                self.send_message_to_receiver_with_attachments(
                    smtp_config,
                    email_id.as_str(),
                    email_subject,
                    html_content,
                    attachments,
                )
                .await?;
            }
        }

        Ok(())
    }

    #[doc = r#"
        인덱스 알람 정보를 텔레그램 메시지로 발송하는 비동기 함수.

        1. 텔레그램 봇 레포지토리와 시스템 설정에서 메시지 청크 크기를 가져온다
        2. `LogIndexResult` 배열을 청크 단위로 분할하여 처리:
           - 청크 크기는 `system_config.message_chunk_size`로 결정
           - 텔레그램 메시지 길이 제한을 고려한 분할 처리
        3. 각 청크별로 알람 메시지 포맷을 구성:
           - 🚨 헤더로 시작하는 알람 메시지
           - 📌 인덱스명과 📊💾🕐 아이콘으로 정보 표시
           - ⚠️ 주의사항으로 마무리
        4. `tele_repo.bot_send`를 통해 각 청크별로 순차 발송
        5. 모든 청크 발송 완료 시 성공 반환

        # Arguments
        * `log_index_results` - 알람 대상 인덱스 정보 배열

        # Returns
        * `Ok(())` - 모든 메시지 발송 성공
        * `Err(anyhow::Error)` - 텔레그램 봇 발송 실패 시

        # Note
        청크 단위 처리를 통해 텔레그램 메시지 길이 제한을 우회하고,
        대량의 인덱스 알람도 안정적으로 발송할 수 있다.
    "#]
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
                msg_format.push_str(&format!("📌 {} 📌\n", log_result.index_name()));

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

    #[doc = r#"
        인덱스 문서 개수 알람을 이메일로 발송하는 함수.

        1. 이메일 제목과 HTML 템플릿을 생성하여 알람 내용을 구성
        2. `generate_index_alert_html`을 통해 인덱스 알람 정보를 HTML 형식으로 변환
        3. SQL Server의 stored procedure를 통해 내부 이메일 시스템(imailer)으로 발송
        4. 각 수신자별로 개별적으로 이메일 발송하며, 실패 시에도 다른 수신자에게는 계속 발송
        5. SMTP 직접 발송 기능은 주석 처리되어 있으며, SP 방식을 사용

        # Arguments
        * `log_index_results` - 알람 대상 인덱스 정보 배열

        # Returns
        * `anyhow::Result<()>` - 발송 프로세스 완료 여부
    "#]
    async fn send_email_index_alert(
        &self,
        log_index_results: &[LogIndexResult],
    ) -> anyhow::Result<()> {
        let elastic_config: &'static ElasticServerConfig = get_mon_elastic_config_info();
        let receiver_email_list: &Vec<ReceiverEmail> = &self.receiver_email_list().emails;

        let email_subject: String =
            String::from("[Elasticsearch] Index Document Count Change Detected");

        let html_content: String =
            self.generate_index_alert_html(log_index_results, elastic_config)?;

        /* SMTP 버전 -> 온라인망 사용용*/
        self.send_message_to_receivers_smtp(&email_subject, &html_content)
            .await?;

        /* SP 버전 */
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

    #[doc = r#"
        인덱스 알람 정보를 HTML 형식의 이메일 템플릿으로 변환하는 함수.

        1. HTML 템플릿 파일(`HTML_TEMPLATE_PATH`)을 읽어온다
        2. `generate_alert_rows`를 통해 인덱스별 알람 데이터를 테이블 행으로 변환
        3. 템플릿 내의 플레이스홀더를 실제 데이터로 교체:
           - `{cluster_name}`: Elasticsearch 클러스터명
           - `{alert_time}`: 현재 시각 (UTC)
           - `{alert_rows}`: 알람 데이터 테이블 행들
        4. 완성된 HTML 문자열을 반환

        # Arguments
        * `log_index_results` - 알람 대상 인덱스 정보 배열
        * `elastic_config` - Elasticsearch 설정 정보

        # Returns
        * `String` - 완성된 HTML 이메일 템플릿
        * `anyhow::Error` - 템플릿 파일 읽기 실패 시
    "#]
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

    #[doc = r#"
        인덱스 알람 정보를 HTML 테이블 행으로 변환하는 함수.

        1. 각 `LogIndexResult`를 순회하면서 HTML 테이블 행 생성
        2. 각 행에는 다음 정보가 포함됨:
           - 인덱스명: `log_result.index_name()`
           - 현재 문서 개수: `log_result.cur_cnt()`
           - 변동률: `log_result.fluctuation_val()`
           - 히스토리 정보: `generate_history_table_html`을 통해 생성된 상세 히스토리
        3. 각 행은 CSS 스타일이 인라인으로 적용된 `<tr>` 태그로 구성
        4. 모든 행을 연결하여 하나의 문자열로 반환

        # Arguments
        * `log_index_results` - 알람 대상 인덱스 정보 배열

        # Returns
        * `String` - HTML 테이블 행들이 연결된 문자열
        * `anyhow::Error` - 처리 실패 시
    "#]
    fn generate_alert_rows(&self, log_index_results: &[LogIndexResult]) -> anyhow::Result<String> {
        let mut rows: String = String::new();

        for log_result in log_index_results {
            if let Some(alert_formats) = log_result.alert_index_format() {
                rows.push_str(&format!(
                    r#"<tr>
                        <td style="border: 1px solid #ddd; padding: 12px; text-align: left; vertical-align: top; background-color: #fff;">{}</td>
                        <td style="border: 1px solid #ddd; padding: 12px; text-align: left; vertical-align: top; background-color: #fff;">{}</td>
                        <td style="border: 1px solid #ddd; padding: 12px; text-align: left; vertical-align: top; background-color: #fff; color: red;">{}%</td>
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

    #[doc = r#"
        알람 인덱스 히스토리 정보를 HTML div 요소들로 변환하는 함수.

        1. `AlertIndex` 배열을 순회하면서 각 항목의 시간과 문서 개수를 추출
        2. 각 항목을 "timestamp -> count" 형식의 div로 변환
        3. 모든 div를 감싸는 컨테이너 div를 생성하여 CSS 스타일 적용:
           - 글자색: #555 (회색)
           - 글자 크기: 14px
           - 줄 간격: 1.5
        4. 완성된 HTML 문자열을 반환

        이 함수는 이메일 테이블 내부에서 각 인덱스의 변동 히스토리를 시각적으로 표현하는데 사용됨.

        # Arguments
        * `alert_indexes` - 알람 인덱스 정보 배열

        # Returns
        * `String` - 히스토리 정보가 포함된 HTML div 컨테이너
    "#]
    fn generate_history_table_html(&self, alert_indexes: &[AlertIndex]) -> String {
        let mut inner_div: String = String::from(r#""#);

        for alert_index in alert_indexes {
            inner_div.push_str(&format!(
                r#"<div>{} -> {}</div>"#,
                alert_index.timestamp(),
                alert_index.cnt()
            ));
        }

        let history_divs: String = String::from(&format!(
            r#"
                <div style="color: #555; font-size: 14px; line-height: 1.5;">
                {}
                </div>
            "#,
            inner_div
        ));

        history_divs
    }
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
    #[doc = r#"
        인덱스 알람 메시지를 텔레그램과 이메일로 동시에 발송하는 메인 함수.

        1. 텔레그램 알람과 이메일 알람을 병렬로 처리하여 성능을 최적화
        2. 각각의 발송 방식이 실패하더라도 다른 방식에는 영향을 주지 않음:
           - 텔레그램 발송 실패 시: 에러 로깅 후 계속 진행
           - 이메일 발송 실패 시: 에러 로깅 후 계속 진행
        3. `tokio::join!`을 사용하여 두 작업을 동시에 실행
        4. 전체 프로세스는 항상 Ok(())를 반환하여 시스템 안정성 보장

        이는 알람 시스템의 핵심 함수로, 다중 채널 알람 발송의 신뢰성을 보장함.

        # Arguments
        * `log_index_results` - 알람 발송 대상 인덱스 정보 배열

        # Returns
        * `Result<(), anyhow::Error>` - 항상 성공을 반환 (개별 실패는 로깅만)
    "#]
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

        Ok(())
    }
}
