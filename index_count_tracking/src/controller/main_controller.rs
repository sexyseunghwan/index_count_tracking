use crate::common::*;

use crate::utils_modules::{io_utils::*, time_utils::*};

use crate::model::{
    configs::total_config::*,
    index::{alert_index::*, index_list_config::*},
    report::{daily_report::*, report_config::*},
};

use crate::dto::log_index_result::*;

use crate::env_configuration::env_config::*;

use crate::traits::service_traits::{
    chart_service::*, daily_report_service::*, notification_service::*, query_service::*,
    tracking_monitor_service::*,
};

#[derive(Debug, new)]
pub struct MainController<T: TrackingMonitorService, D: DailyReportService> {
    tracking_monitor_service: T,
    daily_report_service: D,
}

impl<T: TrackingMonitorService, D: DailyReportService> MainController<T, D> {
    #[doc = r#"
        메인 루프를 실행하는 핵심 함수로, 30초 간격으로 인덱스 모니터링 작업을 반복 수행한다.

        1. 인덱스 설정 파일(`INDEX_LIST_PATH`)을 읽어와 모니터링 대상 인덱스 목록을 가져온다
        2. 30초마다 다음 작업들을 순차적으로 실행:
           - `save_index_cnt_infos`: 각 인덱스의 현재 문서 개수를 모니터링 인덱스에 저장
           - `verify_index_cnt`: 저장된 데이터를 바탕으로 각 인덱스의 문서 개수 변동을 검증
           - `alert_index_status`: 변동이 허용 범위를 초과한 인덱스에 대해 알람 발송
        3. 무한루프로 동작하며, 각 단계에서 오류 발생 시 해당 사이클을 중단하고 다음 사이클로 진행

        # Returns
        * `anyhow::Result<()>` - 정상 종료 시 Ok(()), 치명적 오류 시 Err
    "#]
    pub async fn main_task(&self) -> anyhow::Result<()> {
        let target_index_info_list: IndexListConfig =
            read_toml_from_file::<IndexListConfig>(&INDEX_LIST_PATH)?;
        let mon_index_name: &str = get_system_config_info().monitor_index_name();

        /* 1. 모니터링 테스크 */
        let tracking_monitor_task = self
            .tracking_monitor_service
            .tracking_monitor_loop(mon_index_name, &target_index_info_list);
        /* 2. 리포트 테스크 */
        let daily_report_task = self
            .daily_report_service
            .daily_report_loop(mon_index_name, &target_index_info_list);

        /* 모니터링 태스크와 일일 리포트 태스크를 병렬로 실행 */
        tokio::try_join!(tracking_monitor_task, daily_report_task)?;

        Ok(())
    }
}

// impl<
//     N: NotificationService,
//     TQ: QueryService,
//     MQ: QueryService,
//     DR: DailyReportService,
//     //C: ChartService,
// > MainController<N, TQ, MQ, DR>
// {
//     #[doc = r#"
//         메인 루프를 실행하는 핵심 함수로, 30초 간격으로 인덱스 모니터링 작업을 반복 수행한다.

//         1. 인덱스 설정 파일(`INDEX_LIST_PATH`)을 읽어와 모니터링 대상 인덱스 목록을 가져온다
//         2. 30초마다 다음 작업들을 순차적으로 실행:
//            - `save_index_cnt_infos`: 각 인덱스의 현재 문서 개수를 모니터링 인덱스에 저장
//            - `verify_index_cnt`: 저장된 데이터를 바탕으로 각 인덱스의 문서 개수 변동을 검증
//            - `alert_index_status`: 변동이 허용 범위를 초과한 인덱스에 대해 알람 발송
//         3. 무한루프로 동작하며, 각 단계에서 오류 발생 시 해당 사이클을 중단하고 다음 사이클로 진행

//         # Returns
//         * `anyhow::Result<()>` - 정상 종료 시 Ok(()), 치명적 오류 시 Err
//     "#]
//     pub async fn main_task(&self) -> anyhow::Result<()> {
//         let index_list: IndexListConfig = read_toml_from_file::<IndexListConfig>(&INDEX_LIST_PATH)?;
//         let mon_index_name: &str = get_system_config_info().monitor_index_name();

//         /* 1. 모니터링 테스크 */
//         //let monitoring_task = self.monitoring_loop(&index_list, mon_index_name);
//         /* 2. 리포트 테스크 */
//         //let daily_report_task = self.daily_report_loop(&index_list, mon_index_name);

//         /* 모니터링 태스크와 일일 리포트 태스크를 병렬로 실행 */
//         //tokio::try_join!(monitoring_task, daily_report_task)?;

//         Ok(())
//     }

//     // /********************[모니터링 테스크 관련 함수]************************/
//     // /******************************************************************/
//     // /******************************************************************/
//     // /******************************************************************/
//     // /******************************************************************/
//     // #[doc = ""]
//     // async fn monitoring_loop(
//     //     &self,
//     //     index_list: &IndexListConfig,
//     //     mon_index_name: &str,
//     // ) -> anyhow::Result<()> {

//     //     let mut ticker: Interval = interval(Duration::from_secs(30));

//     //     loop {

//     //         ticker.tick().await;

//     //         /* 1. 인덱스 문서 개수 정보 저장 */
//     //         if let Err(e) = self.save_index_cnt_infos(index_list, mon_index_name).await {
//     //             error!(
//     //                 "[MainController->monitoring_loop] Failed to save index count infos: {:?}",
//     //                 e
//     //             );
//     //             continue;
//     //         }

//     //         /* 2. 인덱스 문서 개수 검증 */
//     //         let index_doc_verification: Vec<LogIndexResult> =
//     //             match self.verify_index_cnt(mon_index_name, index_list).await {
//     //                 Ok(results) => results,
//     //                 Err(e) => {
//     //                     error!(
//     //                         "[MainController->monitoring_loop] Failed to verify index count: {:?}",
//     //                         e
//     //                     );
//     //                     continue;
//     //                 }
//     //             };

//     //         if index_doc_verification.len() > 0 {
//     //             /* 3. 검증 결과를 바탕으로 알람을 보내주는 로직 */
//     //             if let Err(e) = self.alert_index_status(&index_doc_verification).await {
//     //                 error!(
//     //                     "[MainController->monitoring_loop] Failed to send alert: {:?}",
//     //                     e
//     //                 );
//     //             }
//     //         }
//     //     }
//     // }

//     // #[doc = "인덱스 문서 개수 정보 색인 해주는 함수"]
//     // async fn save_index_cnt_infos(
//     //     &self,
//     //     index_list: &IndexListConfig,
//     //     mon_index_name: &str,
//     // ) -> anyhow::Result<()> {
//     //     let cur_utc_time: DateTime<Utc> = Utc::now();

//     //     for index_config in index_list.index() {
//     //         let index_name: &str = index_config.index_name();

//     //         /* 해당 인덱스의 문서 개수 */
//     //         let doc_cnt: usize = match self
//     //             .target_query_service
//     //             .get_index_doc_count(index_config.index_name())
//     //             .await
//     //         {
//     //             Ok(doc_cnt) => doc_cnt,
//     //             Err(e) => {
//     //                 error!("{:?}", e);
//     //                 continue;
//     //             }
//     //         };

//     //         /* 모니터링 인덱스에 해당 인덱스의 문서수를 색인 */
//     //         let alert_index: AlertIndex = AlertIndex::new(
//     //             index_name.to_string(),
//     //             doc_cnt,
//     //             convert_date_to_str(cur_utc_time, Utc),
//     //         );

//     //         /* 해당 정보를 모니터링 클러스터에 색인 */
//     //         self.mon_query_service
//     //             .post_log_index(mon_index_name, &alert_index)
//     //             .await?;
//     //     }

//     //     Ok(())
//     // }

//     // #[doc = "인덱스 문서 개수 검증"]
//     // async fn verify_index_cnt(
//     //     &self,
//     //     mon_index_name: &str,
//     //     index_list: &IndexListConfig,
//     // ) -> anyhow::Result<Vec<LogIndexResult>> {
//     //     let cur_timestamp_utc: DateTime<Utc> = Utc::now();
//     //     let mut log_index_results: Vec<LogIndexResult> = Vec::new();

//     //     for index_config in index_list.index() {
//     //         let log_index_res: LogIndexResult = self
//     //             .mon_query_service
//     //             .get_alert_infos_from_log_index(mon_index_name, index_config, cur_timestamp_utc)
//     //             .await?;

//     //         if log_index_res.alert_yn {
//     //             log_index_results.push(log_index_res);
//     //         }
//     //     }

//     //     Ok(log_index_results)
//     // }

//     // #[doc = "인덱스 문서 개수 이상 상황 알람 발송"]
//     // async fn alert_index_status(&self, log_index_res: &[LogIndexResult]) -> anyhow::Result<()> {
//     //     info!(
//     //         "Sending index alert for {} problematic indices",
//     //         log_index_res.len()
//     //     );

//     //     match self
//     //         .notification_service
//     //         .send_index_alert_message(log_index_res)
//     //         .await
//     //     {
//     //         Ok(_) => {
//     //             info!("Successfully sent index alert notifications");
//     //         }
//     //         Err(e) => {
//     //             error!(
//     //                 "[ERROR][MainController->alert_index_status] Failed to send alert notifications: {:?}",
//     //                 e
//     //             );
//     //             return Err(e);
//     //         }
//     //     }

//     //     Ok(())
//     // }

//     // /*********************[리포트 테스크 관련 함수]*************************/
//     // /******************************************************************/
//     // /******************************************************************/
//     // /******************************************************************/
//     // /******************************************************************/
//     // #[doc = ""]
//     // async fn daily_report_loop(
//     //     &self,
//     //     target_index_info_list: &IndexListConfig,
//     //     mon_index_name: &str,
//     // ) -> anyhow::Result<()> {
//     //     let report_config: &ReportConfig = get_daily_report_config_info();

//     //     /* 데일리 보고용 활성화 여부 */
//     //     if !report_config.enabled {
//     //         info!(
//     //             "[MainController->daily_report_loop] Daily report is disabled. Skipping daily report scheduler."
//     //         );

//     //         /* 무한 대기 (데일리 보고용 기능 비활성화) */
//     //         loop {
//     //             tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
//     //         }
//     //     }

//     //     info!(
//     //         "Starting daily report scheduler with cron schedule: {}",
//     //         report_config.cron_schedule
//     //     );

//     //     /* 크론 스케줄 파싱 */
//     //     let schedule: cron::Schedule = cron::Schedule::from_str(&report_config.cron_schedule)
//     //         .map_err(|e| {
//     //             anyhow!(
//     //                 "[MainController->daily_report_loop] Failed to parse cron schedule '{}': {:?}",
//     //                 report_config.cron_schedule,
//     //                 e
//     //             )
//     //         })?;

//     //     loop {
//     //         /* 보고용 스케쥴은 한국시간 기준으로 한다 GMT+9 */
//     //         let now_local: DateTime<Local> = chrono::Local::now();

//     //         /* 다음 실행 시간 계산 */
//     //         let next_run: DateTime<Local> = schedule
//     //             .upcoming(now_local.timezone())
//     //             .next()
//     //             .ok_or_else(|| anyhow!("[MainController->daily_report_loop] Failed to calculate next run time from cron schedule"))?;

//     //         let duration_until_next_run: Duration = match (next_run - now_local).to_std() {
//     //             Ok(next_run) => next_run,
//     //             Err(e) => {
//     //                 error!("[MainController->daily_report_loop] Failed to calculate duration: {:?}", e);
//     //                 continue;
//     //             }
//     //         };

//     //         info!(
//     //             "Next daily report scheduled at: {}. Sleeping for {:?}",
//     //             next_run.format("%Y-%m-%d %H:%M:%S"),
//     //             duration_until_next_run
//     //         );

//     //         /* thread sleep */
//     //         tokio::time::sleep(duration_until_next_run).await;

//     //         // /* 일일 리포트 생성 및 발송 */
//     //         // info!("Starting daily report generation");

//     //         // /* 한국시간을 UTC 시간으로 변환 */
//     //         // let cur_utc_time: DateTime<Utc> = convert_utc_from_local(now);

//     //         // match self
//     //         //     .generate_and_send_daily_report(index_list, mon_index_name, cur_utc_time)
//     //         //     .await
//     //         // {
//     //         //     Ok(_) => {
//     //         //         info!("Daily report sent successfully");
//     //         //     }
//     //         //     Err(e) => {
//     //         //         error!(
//     //         //             "[MainController->daily_report_loop] Failed to generate/send daily report: {:?}",
//     //         //             e
//     //         //         );
//     //         //     }
//     //         // }

//     //     }
//     // }

//     // #[doc = ""]
//     // async fn generate_and_send_daily_reports(
//     //     &self,
//     //     target_index_info_list: &IndexListConfig,
//     //     mon_index_name: &str,
//     //     now_local: DateTime<Local>
//     // ) -> anyhow::Result<()> {

//     //     /* 데이터는 UTC 기준으로 쌓이기 때문에 UTC로 컨버팅 해준다. */
//     //     let now_utc: DateTime<Utc> = convert_utc_from_local(now_local);

//     //     /* UTC 시간 기준부로 24시간전의 시각을 계산한다. */
//     //     let one_day_prev_cur_utc: DateTime<Utc> = minus_h(now_utc, 24);

//     //     Ok(())
//     // }

//     // #[doc = ""]
//     // async fn generate_and_send_daily_report(
//     //     &self,
//     //     index_list: &IndexListConfig,
//     //     mon_index_name: &str,
//     //     cur_utc_time: DateTime<Utc>,
//     // ) -> anyhow::Result<()> {
//     //     /* 현재 시간 부로 24시간전의 mon_index_name 개수의 모든 데이터를 가져와준다. */
//     //     // let one_day_prev_time: DateTime<Utc> = minus_h(cur_utc_time, 240);

//     //     // let mut chart_img_path_list: Vec<PathBuf> = Vec::new();

//     //     // for index in index_list.index() {
//     //     //     let index_name: &str = index.index_name();

//     //     //     /* 여기서 넘어온 AlertIndex 시각정보는 모두 한국시각으로 전환된 결과임. */
//     //     //     let index_cnt_infos: Vec<AlertIndex> = self
//     //     //         .mon_query_service
//     //     //         .get_report_infos_from_log_index(
//     //     //             mon_index_name,
//     //     //             index_name,
//     //     //             one_day_prev_time,
//     //     //             cur_utc_time,
//     //     //         )
//     //     //         .await?;

//     //     //     let mut x_label: Vec<String> = Vec::new();
//     //     //     let mut y_label: Vec<i64> = Vec::new();

//     //     //     for index_info in index_cnt_infos {
//     //     //         x_label.push(index_info.timestamp);
//     //     //         y_label.push(index_info.cnt as i64);
//     //     //     }

//     //     //     let output_path: PathBuf =
//     //     //         PathBuf::from(format!("./pics/line_chart_{}.png", index_name));

//     //     //     let cur_local_time: DateTime<Local> = convert_local_from_utc(cur_utc_time);
//     //     //     let one_day_prev_local_time: DateTime<Local> =
//     //     //         convert_local_from_utc(one_day_prev_time);

//     //     //     match self
//     //     //         .chart_service
//     //     //         .generate_line_chart(
//     //     //             &format!(
//     //     //                 "[{} ~ {}] {}",
//     //     //                 convert_date_to_str(one_day_prev_local_time, Local),
//     //     //                 convert_date_to_str(cur_local_time, Local),
//     //     //                 index_name
//     //     //             ),
//     //     //             x_label,
//     //     //             y_label,
//     //     //             &output_path,
//     //     //             "timestamp",
//     //     //             "index count",
//     //     //         )
//     //     //         .await {
//     //     //             Ok(_) => (),
//     //     //             Err(e) => {
//     //     //                 error!("[MainController->generate_and_send_daily_report] {:?}", e);
//     //     //                 continue;
//     //     //             }
//     //     //         }

//     //     //     chart_img_path_list.push(output_path);
//     //     // }

//     //     // /**** 아래의 코드는 AI 가 짜준 이쁜 쓰레기 -> 검증이 필요함 ****/
//     //     // /* Daily Report HTML 생성 */
//     //     // let cur_local_time: DateTime<Local> = convert_local_from_utc(cur_utc_time);

//     //     // let email_subject: String = format!(
//     //     //     "[Elasticsearch] Daily Index Report - {}",
//     //     //     convert_date_to_str(cur_local_time, Local)
//     //     // );

//     //     // let html_content: String = self.generate_daily_report_html(
//     //     //     index_list,
//     //     //     cur_local_time,
//     //     // )?;

//     //     // /* Daily Report 이메일 발송 */
//     //     // self.notification_service
//     //     //     .send_daily_report_email(&email_subject, &html_content, &chart_img_path_list)
//     //     //     .await?;

//     //     // info!("Daily report email sent successfully with {} charts", chart_img_path_list.len());

//     //     Ok(())
//     // }

//     // // #[doc = "일일 리포트용 HTML 생성 (템플릿 기반) - 검증중"]
//     // // fn generate_daily_report_html(
//     // //     &self,
//     // //     index_list: &IndexListConfig,
//     // //     report_date: DateTime<Local>,
//     // // ) -> anyhow::Result<String> {

//     // //     /* HTML 템플릿 파일 읽기 */
//     // //     let template_content: String = fs::read_to_string(&*DAILY_REPORT_TEMPLATE_PATH)
//     // //         .map_err(|e| {
//     // //             anyhow!(
//     // //                 "[MainController->generate_daily_report_html] Failed to read template: {:?}",
//     // //                 e
//     // //             )
//     // //         })?;

//     // //     /* 템플릿 플레이스홀더 교체 */
//     // //     let html_content: String = template_content
//     // //         .replace("{{REPORT_DATE}}", &convert_date_to_str(report_date, Local))
//     // //         .replace("{{TOTAL_INDICES}}", &index_list.index().len().to_string())
//     // //         .replace("{{TOTAL_DOCS_START}}", "N/A") // 추후 실제 데이터로 교체 가능
//     // //         .replace("{{TOTAL_DOCS_END}}", "N/A")
//     // //         .replace("{{TOTAL_CHANGE}}", "N/A")
//     // //         .replace("{{CHANGE_STYLE}}", "")
//     // //         .replace("{{INDICES_WITH_ALERTS}}", "0")
//     // //         .replace("{{TOTAL_ALERTS}}", "0")
//     // //         .replace("{{INDEX_ROWS}}", &self.generate_index_rows(index_list));

//     // //     Ok(html_content)
//     // // }

//     // // #[doc = "인덱스별 상세 정보 테이블 행 생성"]
//     // // fn generate_index_rows(&self, index_list: &IndexListConfig) -> String {
//     // //     let mut rows: String = String::new();

//     // //     for index_config in index_list.index() {
//     // //         rows.push_str(&format!(
//     // //             r#"<tr>
//     // //                 <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
//     // //                 <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">-</td>
//     // //                 <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">-</td>
//     // //                 <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">-</td>
//     // //                 <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">-</td>
//     // //                 <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">0</td>
//     // //                 <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">✅ 정상</td>
//     // //             </tr>"#,
//     // //             index_config.index_name()
//     // //         ));
//     // //     }

//     // //     rows
//     // // }
// }
