use crate::common::*;
use crate::env_configuration::env_config::*;
use crate::model::configs::total_config::get_system_config_info;
use crate::model::{index::index_list_config::*, report::daily_report::*};
use crate::repository::sqlserver_repository_impl::*;
use crate::traits::repository_traits::sqlserver_repository::*;
use crate::traits::service_traits::{
    daily_report_service::*, notification_service::*, query_service::*,
};
use crate::utils_modules::{io_utils::*, time_utils::*};

#[derive(Debug, new)]
pub struct DailyReportServiceImpl<Q: QueryService> {
    query_service: Q,
}

#[async_trait]
impl<Q: QueryService + Sync> DailyReportService for DailyReportServiceImpl<Q> {
    //     #[doc = ""]
    //     async fn generate_daily_report(
    //         &self,
    //         index_list: &IndexListConfig,
    //         mon_index_name: &str,
    //     ) -> anyhow::Result<DailyReport> {
    //         let today: String = get_current_utc_naivedatetime_str();
    //         let yesterday: String = calc_time_window(&today, 86400)?; /* 24시간 전 */
    //         info!("Generating daily report from {} to {}", yesterday, today);

    //         let mut index_stats: Vec<IndexDailyStats> = Vec::new();

    //         for index_config in index_list.index() {
    //             let index_name: &String = index_config.index_name();

    //             let start_count: usize = match self
    //                 .get_index_count_at_time(mon_index_name, index_name, &yesterday)
    //                 .await
    //             {
    //                 Ok(count) => count,
    //                 Err(e) => {
    //                     error!("Failed to get start count for {}: {:?}", index_name, e);
    //                     0
    //                 }
    //             };

    //             let end_count: usize = match self
    //                 .get_index_count_at_time(mon_index_name, index_name, &today)
    //                 .await
    //             {
    //                 Ok(count) => count,
    //                 Err(e) => {
    //                     error!("Failed to get end count for {}: {:?}", index_name, e);
    //                     0
    //                 }
    //             };

    //             let alert_count: usize = match self
    //                 .get_alert_count_in_period(mon_index_name, index_name, &yesterday, &today)
    //                 .await
    //             {
    //                 Ok(count) => count,
    //                 Err(e) => {
    //                     error!("Failed to get alert count for {}: {:?}", index_name, e);
    //                     0
    //                 }
    //             };

    //             index_stats.push(IndexDailyStats::new(
    //                 index_name.to_string(),
    //                 start_count,
    //                 end_count,
    //                 alert_count,
    //             ));
    //         }

    //         let summary = ReportSummary::calculate(&index_stats);

    //         Ok(DailyReport {
    //             date: today,
    //             summary,
    //             index_stats,
    //         })
    //     }

    //     async fn send_daily_report_email(&self, report: &DailyReport) -> anyhow::Result<()> {
    //         let html_content = self.generate_html_report(report)?;
    //         let subject = format!(
    //             "[인덱스 모니터링] 일일 리포트 - {}",
    //             &report.date[0..10] // YYYY-MM-DD 부분만 추출
    //         );

    //         // SQL Server SP를 통한 이메일 발송 (기존 방식과 동일)
    //         let sql_conn: Arc<SqlServerRepositoryImpl> = get_sqlserver_repo();

    //         // 이메일 수신자 목록을 파일에서 읽어옴
    //         let receiver_email_config: crate::model::configs::receiver_email_config::ReceiverEmailConfig = read_toml_from_file::<crate::model::configs::receiver_email_config::ReceiverEmailConfig>(&EMAIL_RECEIVER_PATH)
    //             .map_err(|e| anyhow!("Failed to read receiver email config: {:?}", e))?;

    //         for receiver in &receiver_email_config.emails {
    //             match sql_conn
    //                 .execute_imailer_procedure(receiver.email_id(), &subject, &html_content)
    //                 .await
    //             {
    //                 Ok(_) => {
    //                     info!("Daily report email sent successfully to: {}", receiver.email_id());
    //                 }
    //                 Err(e) => {
    //                     error!("Failed to send daily report email to {}: {:?}", receiver.email_id(), e);
    //                 }
    //             }
    //         }

    //         Ok(())
    //     }

    //     async fn get_index_count_at_time(
    //         &self,
    //         mon_index_name: &str,
    //         index_name: &str,
    //         timestamp: &str,
    //     ) -> anyhow::Result<usize> {
    //         let query = serde_json::json!({
    //             "query": {
    //                 "bool": {
    //                     "must": [
    //                         {
    //                             "term": {
    //                                 "index_name.keyword": index_name
    //                             }
    //                         },
    //                         {
    //                             "range": {
    //                                 "timestamp": {
    //                                     "lte": timestamp
    //                                 }
    //                             }
    //                         }
    //                     ]
    //                 }
    //             },
    //             "sort": [
    //                 {
    //                     "timestamp": {
    //                         "order": "desc"
    //                     }
    //                 }
    //             ],
    //             "size": 1
    //         });

    //         let response = self
    //             .query_service
    //             .execute_search_query(mon_index_name, &query)
    //             .await?;

    //         if let Some(hits) = response["hits"]["hits"].as_array() {
    //             if let Some(hit) = hits.first() {
    //                 if let Some(doc_count) = hit["_source"]["doc_count"].as_u64() {
    //                     return Ok(doc_count as usize);
    //                 }
    //             }
    //         }

    //         Ok(0)
    //     }

    //     async fn get_alert_count_in_period(
    //         &self,
    //         mon_index_name: &str,
    //         index_name: &str,
    //         start_time: &str,
    //         end_time: &str,
    //     ) -> anyhow::Result<usize> {
    //         let query = serde_json::json!({
    //             "query": {
    //                 "bool": {
    //                     "must": [
    //                         {
    //                             "term": {
    //                                 "index_name.keyword": index_name
    //                             }
    //                         },
    //                         {
    //                             "range": {
    //                                 "timestamp": {
    //                                     "gte": start_time,
    //                                     "lte": end_time
    //                                 }
    //                             }
    //                         }
    //                     ]
    //                 }
    //             },
    //             "size": 0
    //         });

    //         let response = self
    //             .query_service
    //             .execute_search_query(mon_index_name, &query)
    //             .await?;

    //         if let Some(total) = response["hits"]["total"]["value"].as_u64() {
    //             return Ok(total as usize);
    //         }

    //         Ok(0)
    //     }
    // }

    // impl<Q: QueryService + Sync> DailyReportServiceImpl<Q> {
    //     fn generate_html_report(&self, report: &DailyReport) -> anyhow::Result<String> {
    //         // 템플릿 파일 읽기
    //         let template_path = "templates/daily_report.html";
    //         let mut template = std::fs::read_to_string(template_path)
    //             .map_err(|e| anyhow!("Failed to read template file {}: {:?}", template_path, e))?;

    //         // 기본 변수 치환
    //         template = template.replace("{{REPORT_DATE}}", &report.date[0..19]);
    //         template = template.replace("{{TOTAL_INDICES}}", &report.summary.total_indices.to_string());
    //         template = template.replace("{{TOTAL_DOCS_START}}", &report.summary.total_documents_start.to_string());
    //         template = template.replace("{{TOTAL_DOCS_END}}", &report.summary.total_documents_end.to_string());
    //         template = template.replace("{{TOTAL_CHANGE}}", &format!("{:+}", report.summary.total_change));
    //         template = template.replace("{{INDICES_WITH_ALERTS}}", &report.summary.indices_with_alerts.to_string());
    //         template = template.replace("{{TOTAL_ALERTS}}", &report.summary.total_alerts.to_string());

    //         // 변동량 스타일 적용
    //         let change_style = if report.summary.total_change > 0 {
    //             "color: #28a745;"
    //         } else if report.summary.total_change < 0 {
    //             "color: #dc3545;"
    //         } else {
    //             "color: #6c757d;"
    //         };
    //         template = template.replace("{{CHANGE_STYLE}}", change_style);

    //         // 인덱스별 행 생성
    //         let mut index_rows = String::new();
    //         for (i, stat) in report.index_stats.iter().enumerate() {
    //             let row_style = if i % 2 == 1 {
    //                 "background-color: #f8f9fa;"
    //             } else {
    //                 "background-color: white;"
    //             };

    //             let (status_color, status_text) = match stat.status {
    //                 IndexStatus::Normal => ("#28a745", "정상"),
    //                 IndexStatus::Warning => ("#ffc107", "주의"),
    //                 IndexStatus::Critical => ("#dc3545", "위험"),
    //             };

    //             let change_color = if stat.change > 0 {
    //                 "#28a745"
    //             } else if stat.change < 0 {
    //                 "#dc3545"
    //             } else {
    //                 "#6c757d"
    //             };

    //             index_rows.push_str(&format!(
    //                 "<tr style=\"{}\">
    //                     <td style=\"border: 1px solid #ddd; padding: 12px;\">{}</td>
    //                     <td style=\"border: 1px solid #ddd; padding: 12px; font-family: 'Courier New', monospace;\">{}</td>
    //                     <td style=\"border: 1px solid #ddd; padding: 12px; font-family: 'Courier New', monospace;\">{}</td>
    //                     <td style=\"border: 1px solid #ddd; padding: 12px; font-family: 'Courier New', monospace; color: {}; font-weight: bold;\">{:+}</td>
    //                     <td style=\"border: 1px solid #ddd; padding: 12px; font-family: 'Courier New', monospace;\">{:.2}%</td>
    //                     <td style=\"border: 1px solid #ddd; padding: 12px; font-family: 'Courier New', monospace;\">{}</td>
    //                     <td style=\"border: 1px solid #ddd; padding: 12px; color: {}; font-weight: bold;\">{}</td>
    //                 </tr>",
    //                 row_style,
    //                 stat.index_name,
    //                 stat.start_count,
    //                 stat.end_count,
    //                 change_color,
    //                 stat.change,
    //                 stat.change_percentage,
    //                 stat.alert_count,
    //                 status_color,
    //                 status_text
    //             ));
    //         }

    //         template = template.replace("{{INDEX_ROWS}}", &index_rows);

    //         Ok(template)
    //     }
}
