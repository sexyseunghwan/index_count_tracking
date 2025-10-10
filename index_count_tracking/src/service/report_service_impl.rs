use crate::common::*;
use crate::env_configuration::env_config::*;
use crate::model::configs::total_config::get_system_config_info;
use crate::model::{index::index_list_config::*, report::daily_report::*};
use crate::repository::sqlserver_repository_impl::*;
use crate::traits::repository_traits::sqlserver_repository::*;
use crate::traits::service_traits::{
    chart_service::*, report_service::*, notification_service::*, notification_service::*,
    query_service::*,
};
use crate::utils_modules::{io_utils::*, time_utils::*};

use crate::model::{
    configs::total_config::*,
    index::{alert_index::*, index_list_config::*},
    report::{daily_report::*, report_config::*},
};

use crate::enums::report_type::*;

#[derive(Debug, new)]
pub struct ReportServiceImpl<Q: QueryService, C: ChartService, N: NotificationService> {
    query_service: Q,
    chart_service: C,
    notification_service: Arc<N>,
}


impl<Q, C, N> ReportServiceImpl<Q, C, N>
where
    Q: QueryService + Sync,
    C: ChartService,
    N: NotificationService,
{
    #[doc = ""]
    async fn report_index_cnt_task(
        &self,
        mon_index_name: &str,
        target_index_info_list: &IndexListConfig,
        local_time: DateTime<Local>,
        hour: i64
    ) -> anyhow::Result<()> {
        let mut img_paths: Vec<PathBuf> = Vec::new();

        /* 여기에 코드를 넣어주는게 맞아보이는데? -> 그래프를 가져오기 위함 */
        for index in target_index_info_list.index() {
            /* 1. 그래프 생성 */
            let graph_path: PathBuf = match self
                .generate_index_history_graph(mon_index_name, index.index_name(), local_time, hour)
                .await
            {
                Ok(graph_path) => graph_path,
                Err(e) => {
                    error!("{:?}", e);
                    continue;
                }
            };

            img_paths.push(graph_path);
        }

        /* Report HTML 생성 */
        let email_subject: String = format!(
            "[Elasticsearch] Index Report - {}",
            convert_date_to_str(local_time, Local)
        );

        let html_content: String =
            self.generate_daily_report_html(target_index_info_list, local_time)?;

        self.notification_service
            .send_daily_report_email(&email_subject, &html_content, &img_paths)
            .await?;

        Ok(())
    }

    #[doc = "일일 리포트용 HTML 생성 (템플릿 기반) - 검증중"]
    fn generate_daily_report_html(
        &self,
        index_list: &IndexListConfig,
        report_date: DateTime<Local>,
    ) -> anyhow::Result<String> {
        /* HTML 템플릿 파일 읽기 */
        let template_content: String =
            fs::read_to_string(&*DAILY_REPORT_TEMPLATE_PATH).map_err(|e| {
                anyhow!(
                    "[MainController->generate_daily_report_html] Failed to read template: {:?}",
                    e
                )
            })?;

        /* 템플릿 플레이스홀더 교체 */
        let html_content: String = template_content
            .replace("{{REPORT_DATE}}", &convert_date_to_str(report_date, Local))
            .replace("{{TOTAL_INDICES}}", &index_list.index().len().to_string())
            .replace("{{TOTAL_DOCS_START}}", "N/A") // 추후 실제 데이터로 교체 가능
            .replace("{{TOTAL_DOCS_END}}", "N/A")
            .replace("{{TOTAL_CHANGE}}", "N/A")
            .replace("{{CHANGE_STYLE}}", "")
            .replace("{{INDICES_WITH_ALERTS}}", "0")
            .replace("{{TOTAL_ALERTS}}", "0")
            .replace("{{INDEX_ROWS}}", &self.generate_index_rows(index_list));

        Ok(html_content)
    }

    #[doc = "인덱스별 상세 정보 테이블 행 생성"]
    fn generate_index_rows(&self, index_list: &IndexListConfig) -> String {
        let mut rows: String = String::new();

        for index_config in index_list.index() {
            rows.push_str(&format!(
                r#"<tr>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">-</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">-</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">-</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">-</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">0</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">✅ 정상</td>
                </tr>"#,
                index_config.index_name()
            ));
        }

        rows
    }

    #[doc = ""]
    async fn generate_index_history_graph(
        &self,
        mon_index_name: &str,
        index_name: &str,
        local_time: DateTime<Local>,
        hour: i64
    ) -> anyhow::Result<PathBuf> {
        let prev_local_time: DateTime<Local> = minus_h_local(local_time, hour);
        let utc_time: DateTime<Utc> = convert_utc_from_local(local_time);
        let prev_utc_time: DateTime<Utc> = minus_h(utc_time, hour);

        /* elasticsearch query 집계 */
        let index_cnt_history: Vec<AlertIndex> = self
            .query_service
            .get_report_infos_from_log_index(mon_index_name, index_name, prev_utc_time, utc_time)
            .await?;

        let mut x_label: Vec<String> = Vec::new();
        let mut y_label: Vec<i64> = Vec::new();

        for index_info in index_cnt_history {
            x_label.push(index_info.timestamp);
            y_label.push(index_info.cnt as i64);
        }

        let output_path: PathBuf =
            PathBuf::from(format!("./pics/{}_line_chart_{}.png", hour, index_name)); 
        
        self.chart_service
            .generate_line_chart(
                &format!(
                    "[{} ~ {}] {}",
                    convert_date_to_str(prev_local_time, Local),
                    convert_date_to_str(local_time, Local),
                    index_name
                ),
                x_label,
                y_label,
                &output_path,
                "timestamp",
                "index count",
            )
            .await?;
        
        Ok(output_path)
    }
}

#[async_trait]
impl<Q, C, N> DailyReportService for DailyReportServiceImpl<Q, C, N>
where
    Q: QueryService + Sync,
    C: ChartService,
    N: NotificationService,
{
    #[doc = ""]
    async fn report_loop(
        &self,
        mon_index_name: &str,
        target_index_info_list: &IndexListConfig,
        report_type: ReportType
    ) -> anyhow::Result<()> {
        
        let report_config: &ReportConfig = match report_type {
            ReportType::OneDay => get_daily_report_config_info(),
            ReportType::OneWeek => get_weekly_report_config_info(),
            ReportType::OneMonth => get_monthly_report_config_info(),
            ReportType::OneYear => get_yearly_report_config_info(),
        };

        /* 데일리 보고용 활성화 여부 */
        if !report_config.enabled {
            info!(
                "[MainController->daily_report_loop] Daily report is disabled. Skipping daily report scheduler."
            );
            
            /* 무한 대기 (데일리 보고용 기능 비활성화) */
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        }

        /* 크론 스케줄 파싱 */
        let schedule: cron::Schedule = cron::Schedule::from_str(&report_config.cron_schedule)
            .map_err(|e| {
                anyhow!(
                    "[MainController->daily_report_loop] Failed to parse cron schedule '{}': {:?}",
                    report_config.cron_schedule,
                    e
                )
            })?;

        info!(
            "Starting daily report scheduler with cron schedule: {}",
            report_config.cron_schedule
        );

        let hour: i64 = get_days(report_type) * 24;

        loop {
            /* 보고용 스케쥴은 한국시간 기준으로 한다 GMT+9 */
            let now_local: DateTime<Local> = chrono::Local::now();

            match self.report_index_cnt_task(mon_index_name, target_index_info_list, now_local, hour).await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);

                } 
            }
            
            /* 다음 실행 시간 계산 */
            let next_run: DateTime<Local> = schedule
                .upcoming(now_local.timezone())
                .next()
                .ok_or_else(|| anyhow!("[MainController->daily_report_loop] Failed to calculate next run time from cron schedule"))?;

            let duration_until_next_run: Duration = match (next_run - now_local).to_std() {
                Ok(next_run) => next_run,
                Err(e) => {
                    error!(
                        "[MainController->daily_report_loop] Failed to calculate duration: {:?}",
                        e
                    );
                    continue;
                }
            };

            info!(
                "Next daily report scheduled at: {}. Sleeping for {:?}",
                next_run.format("%Y-%m-%d %H:%M:%S"),
                duration_until_next_run
            );

            /* thread sleep */
            tokio::time::sleep(duration_until_next_run).await;
        }
    }
}
