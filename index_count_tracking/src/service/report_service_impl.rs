use crate::common::*;
use crate::env_configuration::env_config::*;
use crate::model::index::{self, index_config::*, index_list_config::*};
use crate::traits::service_traits::{
    chart_service::*, notification_service::*, query_service::*, report_service::*,
};
use crate::utils_modules::time_utils::*;

use crate::model::{configs::total_config::*, index::alert_index::*, report::report_config::*};

use crate::dto::{
    alarm::{
        alarm_report_infos::*,
        alarm_index_detail_info::*
    }, 
    index_count_agg_result::*
};

use crate::enums::{report_type::*, index_status::*};

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
        alarm_index_name: &str,
        target_index_info_list: &IndexListConfig,
        local_time: DateTime<Local>,
        report_type: ReportType,
    ) -> anyhow::Result<()> {
        /* 집계 이미지 경로 벡터 */
        let mut img_paths: Vec<PathBuf> = Vec::new();
        
        let hour: i64 = get_days(report_type) * 24;
        let prev_local_time: DateTime<Local> = minus_h_local(local_time, hour);
        let utc_from_local: DateTime<Utc> = convert_utc_from_local(local_time);
        let prev_hour_utc_time: DateTime<Utc> = minus_h(utc_from_local, hour);

        /* 그래프를 가져오기 위함 */
        for index in target_index_info_list.index() {
            /* 1. 그래프 생성 */
            let graph_path: PathBuf = match self
                .generate_index_history_graph(
                    mon_index_name,
                    index.index_name(),
                    local_time,
                    prev_local_time,
                    utc_from_local,
                    prev_hour_utc_time,
                    hour,
                )
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

        /* 시작 시점 총 문서 정보*/
        let start_time_all_index_info: Vec<IndexCountAggResult> = self
            .query_service
            .get_start_time_all_indicies_count(mon_index_name, prev_hour_utc_time, utc_from_local)
            .await?;

        let start_time_all_index_cnt: usize = start_time_all_index_info.iter().map(|x| x.cnt()).sum();
        
        /* 종료 시점 총 문서 정보 */
        let end_time_all_index_info: Vec<IndexCountAggResult> = self
            .query_service
            .get_start_time_all_indicies_count(mon_index_name, prev_hour_utc_time, utc_from_local)
            .await?;

        let end_time_all_index_cnt: usize = end_time_all_index_info.iter().map(|x| x.cnt()).sum();

        /* 총 변동량 -> |시작시점 총 문서 수 - 종료시점 총 문서 수| */
        let total_difference: usize =
            self.calc_start_end_index_cnt(&start_time_all_index_info, &end_time_all_index_info);

        let alarm_report_infos: AlarmReportInfos = self
            .query_service
            .get_index_name_aggregations(alarm_index_name, prev_hour_utc_time, utc_from_local)
            .await?;

        /* 알람 발생 인덱스 개수 */
        let alaram_index_cnt: u64 = alarm_report_infos.distinct_count_u64;

        /* 총 알람 수 */
        let total_alarm_cnt: u64 = alarm_report_infos.buckets().iter().map(|x| x.count).sum();
        
        /* 알람인덱스 상세 정보 */
        //let alarm_index_details = 

        let html_content: String =
            self.generate_daily_report_html(
                target_index_info_list, 
                local_time, 
                report_type,
                start_time_all_index_cnt,
                end_time_all_index_cnt,
                total_difference,
                alaram_index_cnt,
                total_alarm_cnt
            )?;

        /* 이메일로 리포트를 보내줌 */
        self.notification_service
            .send_daily_report_email(&email_subject, &html_content, &img_paths)
            .await?;

        Ok(())
    }

    #[doc = "Elasticsearch 에서 집계하여 그래프를 생성하고 이미지를 저장한 뒤 해당 이미지의 경로를 리턴해주는 함수"]
    async fn generate_index_history_graph(
        &self,
        mon_index_name: &str,
        index_name: &str,
        local_time: DateTime<Local>,
        prev_local_time: DateTime<Local>,
        utc_time: DateTime<Utc>,
        prev_utc_time: DateTime<Utc>,
        hour: i64,
    ) -> anyhow::Result<PathBuf> {
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

    #[doc = ""]
    fn calc_start_end_index_cnt(
        &self,
        start_time_all_index_cnt: &Vec<IndexCountAggResult>,
        end_time_all_index_cnt: &Vec<IndexCountAggResult>,
    ) -> usize {
        let start_total_cnt: usize = start_time_all_index_cnt.iter().map(|x| x.cnt).sum();
        let end_total_cnt: usize = end_time_all_index_cnt.iter().map(|x| x.cnt).sum();

        start_total_cnt.abs_diff(end_total_cnt)
    }
    
    #[doc = ""]
    fn generate_alarm_index_details(
        &self,
        index: &Vec<IndexConfig>,
        start_time_all_index_info: &Vec<IndexCountAggResult>,
        end_time_all_index_info: &Vec<IndexCountAggResult>,
        alarm_report_infos: AlarmReportInfos
    ) {

        let mut alarm_index_details: Vec<AlarmIndexDetailInfo> = Vec::new();

        for index in index {

            let index_name: &str = index.index_name();
            
            let filtered_start_cnt: usize = match start_time_all_index_info
                .iter()
                .find(|item| item.index_name == index_name)
            {
                Some(item) => item.cnt,
                None => {
                    error!("Index not found in start_time_all_index_info: {}", index_name);
                    continue;
                }
            };

            let filtered_end_cnt: usize = match end_time_all_index_info
                .iter()
                .find(|item| item.index_name == index_name) 
            {
                Some(item) => item.cnt,
                None => {
                    error!("Index not found in end_time_all_index_info: {}", index_name);
                    continue;
                }
            };
            
            let difference: usize = filtered_start_cnt.abs_diff(filtered_end_cnt);
            let difference_percent = (difference / filtered_start_cnt) * 100;

            let filtered_alarm_cnt: u64 = match alarm_report_infos.buckets()
                .iter()
                .find(|item| item.name == index_name)
            {
                Some(item) => item.count,
                None => {
                    error!("Index not found in filtered_alarm_cnt: {}", index_name);
                    continue;
                }
            };
            
            let alaram_status: IndexStatus = if filtered_alarm_cnt > 0 {
                IndexStatus::Abnormal
            } else {
                IndexStatus::Normal
            };

            // let alarm_index_detail: AlarmIndexDetailInfo = AlarmIndexDetailInfo::new(
            //     index_name.to_string(), 
            //     filtered_start_cnt, 
            //     filtered_end_cnt, 
            //     difference, 
            //     difference_percent, 
            //     alarm_cnt, 
            //     status
            // );

        }
        

    }

    #[doc = "일일 리포트용 HTML 생성 (템플릿 기반) - 검증중"]
    fn generate_daily_report_html(
        &self,
        index_list: &IndexListConfig,
        report_date: DateTime<Local>,
        report_type: ReportType,
        start_time_all_index_cnt: usize,
        end_time_all_index_cnt: usize,
        total_difference: usize,
        alaram_index_cnt: u64,
        total_alarm_cnt: u64
    ) -> anyhow::Result<String> {
        /* HTML 템플릿 파일 읽기 */
        let template_content: String =
            fs::read_to_string(&*DAILY_REPORT_TEMPLATE_PATH).map_err(|e| {
                anyhow!(
                    "[MainController->generate_daily_report_html] Failed to read template: {:?}",
                    e
                )
            })?;

        let report_name: &str = match report_type {
            ReportType::OneDay => "일일",
            ReportType::OneWeek => "주간",
            ReportType::OneMonth => "월간",
            ReportType::OneYear => "연간",
        };
        
        /* 템플릿 플레이스홀더 교체 */
        let html_content: String = template_content
            .replace("{{REPORT_TYPE}}", report_name)
            .replace("{{REPORT_DATE}}", &convert_date_to_str(report_date, Local))
            .replace("{{TOTAL_INDICES}}", &index_list.index().len().to_string())
            .replace("{{TOTAL_DOCS_START}}", &start_time_all_index_cnt.to_string())
            .replace("{{TOTAL_DOCS_END}}", &end_time_all_index_cnt.to_string())
            .replace("{{TOTAL_CHANGE}}", &total_difference.to_string())
            .replace("{{CHANGE_STYLE}}", "")
            .replace("{{INDICES_WITH_ALERTS}}", &alaram_index_cnt.to_string())
            .replace("{{TOTAL_ALERTS}}", &total_alarm_cnt.to_string())
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
}

#[async_trait]
impl<Q, C, N> ReportService for ReportServiceImpl<Q, C, N>
where
    Q: QueryService + Sync,
    C: ChartService,
    N: NotificationService,
{
    #[doc = "리포트 서비스를 제공해주는 함수"]
    async fn report_loop(
        &self,
        mon_index_name: &str,
        alarm_index_name: &str,
        target_index_info_list: &IndexListConfig,
        report_type: ReportType,
    ) -> anyhow::Result<()> {
        /* 리포트 타입이 어떤 타입인지 확인 */
        let report_config: &ReportConfig = match report_type {
            ReportType::OneDay => get_daily_report_config_info(),
            ReportType::OneWeek => get_weekly_report_config_info(),
            ReportType::OneMonth => get_monthly_report_config_info(),
            ReportType::OneYear => get_yearly_report_config_info(),
        };

        /* 해당 타입 보고용 활성화 여부 */
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

        loop {
            /* 보고용 스케쥴은 한국시간 기준으로 한다 GMT+9 */
            let now_local: DateTime<Local> = chrono::Local::now();

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
                next_run.format("%Y-%m-%dT%H:%M:%S"),
                duration_until_next_run
            );

            /* thread sleep */
            /* tokio::time::sleep(duration_until_next_run).await; */
            let wake: Instant = Instant::now() + duration_until_next_run;
            sleep_until(wake).await;

            /* 메일 보내주는 시각이 되면 함수가 동작함 */
            self.report_index_cnt_task(
                mon_index_name,
                alarm_index_name,
                target_index_info_list,
                now_local,
                report_type,
            )
            .await
            .unwrap_or_else(|e| {
                error!("{:?}", e);
            });
        }
    }
}
