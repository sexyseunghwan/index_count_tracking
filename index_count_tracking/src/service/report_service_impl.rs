use crate::common::*;
use crate::env_configuration::env_config::*;
use crate::model::index::{index_config::*, index_list_config::*};
use crate::traits::service_traits::{
    chart_service::*, notification_service::*, query_service::*, report_service::*,
};
use crate::utils_modules::time_utils::*;

use crate::model::{configs::total_config::*, index::alert_index::*, report::report_config::*};

use crate::dto::{
    alarm::{
        alarm_image_info::*, alarm_index_detail_info::*, alarm_index_diff_detail_infos::*,
        alarm_report_infos::*,
    },
    index_count_agg_result::*,
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
    #[doc = "Function that organizes index change information into a report format and then sends a notification to the administrator."]
    async fn report_index_cnt_task(
        &self,
        mon_index_name: &str,
        alarm_index_name: &str,
        target_index_info_list: &IndexListConfig,
        local_time: DateTime<Local>,
        report_type: ReportType,
    ) -> anyhow::Result<()> {
        let mut alarm_image_infos: Vec<AlarmImageInfo> = Vec::new();

        let hour: i64 = get_days(report_type) * 24;
        let prev_local_time: DateTime<Local> = minus_h_local(local_time, hour);
        let utc_from_local: DateTime<Utc> = convert_utc_from_local(local_time);
        let prev_hour_utc_time: DateTime<Utc> = minus_h(utc_from_local, hour);

        for index in target_index_info_list.index() {
            /* Gecerate Report graph ,*/
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

            let alarm_image_info: AlarmImageInfo =
                AlarmImageInfo::new(index.index_name().to_string(), graph_path);

            alarm_image_infos.push(alarm_image_info);
        }

        /* Generate Report HTML */
        let email_subject: String = format!(
            "[Elasticsearch] Index Report - {}",
            convert_date_to_str(local_time, Local)
        );

        /* Total document information at start time. */
        let start_time_all_index_info: Vec<IndexCountAggResult> = self
            .query_service
            .get_start_time_all_indicies_count(mon_index_name, prev_hour_utc_time, utc_from_local)
            .await?;

        let start_time_all_index_cnt: usize =
            start_time_all_index_info.iter().map(|x| x.cnt()).sum();

        /* Total document information at end time. */
        let end_time_all_index_info: Vec<IndexCountAggResult> = self
            .query_service
            .get_end_time_all_indicies_count(mon_index_name, prev_hour_utc_time, utc_from_local)
            .await?;

        let end_time_all_index_cnt: usize = end_time_all_index_info.iter().map(|x| x.cnt()).sum();

        /* Total variation -> |Total number of documents at start - Total number of documents at end| */
        let total_difference: usize =
            self.calc_start_end_index_cnt(&start_time_all_index_info, &end_time_all_index_info);

        let alarm_report_infos: AlarmReportInfos = self
            .query_service
            .get_index_name_aggregations(alarm_index_name, prev_hour_utc_time, utc_from_local)
            .await?;

        /* Number of alarm occurrence indices. */
        let alaram_index_cnt: u64 = alarm_report_infos.distinct_count_u64;

        /* Total number of alarms */
        let total_alarm_cnt: u64 = alarm_report_infos.buckets().iter().map(|x| x.count).sum();

        /* Detailed information by index */
        let alarm_index_details: Vec<AlarmIndexDetailInfo> = self.generate_alarm_index_details(
            target_index_info_list.index(),
            alarm_report_infos,
            start_time_all_index_info,
            end_time_all_index_info,
        );

        /* Detailed information by index - Maximum change information */
        let alarm_index_diff_detilas: Vec<AlarmIndexDiffDetailInfo> = self
            .generate_alram_index_diff_details(
                target_index_info_list.index(),
                mon_index_name,
                prev_hour_utc_time,
                utc_from_local,
            )
            .await?;

        let html_content: String = self.generate_daily_report_html(
            target_index_info_list,
            prev_local_time,
            local_time,
            report_type,
            start_time_all_index_cnt,
            end_time_all_index_cnt,
            total_difference,
            alaram_index_cnt,
            total_alarm_cnt,
            alarm_index_details,
            alarm_index_diff_detilas,
        )?;

        /* Send the report via email. */
        self.notification_service
            .send_report_information_by_email(&email_subject, &html_content, &alarm_image_infos)
            .await?;

        Ok(())
    }

    #[doc = "Function that aggregates data from Elasticsearch, generates a graph, saves it as an image, and returns the image path."]
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

    fn calc_start_end_index_cnt(
        &self,
        start_time_all_index_cnt: &[IndexCountAggResult],
        end_time_all_index_cnt: &[IndexCountAggResult],
    ) -> usize {
        let start_total_cnt: usize = start_time_all_index_cnt.iter().map(|x| x.cnt).sum();
        let end_total_cnt: usize = end_time_all_index_cnt.iter().map(|x| x.cnt).sum();

        start_total_cnt.abs_diff(end_total_cnt)
    }

    fn generate_alarm_index_details(
        &self,
        index_list: &Vec<IndexConfig>,
        alarm_report_infos: AlarmReportInfos,
        start_time_all_index_info: Vec<IndexCountAggResult>,
        end_time_all_index_info: Vec<IndexCountAggResult>,
    ) -> Vec<AlarmIndexDetailInfo> {
        let mut alarm_index_details: Vec<AlarmIndexDetailInfo> = Vec::new();

        for index in index_list {
            let index_name: &str = index.index_name();

            let filtered_start_cnt: usize = match start_time_all_index_info
                .iter()
                .find(|item| item.index_name == index_name)
            {
                Some(item) => item.cnt,
                None => {
                    error!(
                        "[ReportServiceImpl->generate_alarm_index_details] Index not found in start_time_all_index_info: {}",
                        index_name
                    );
                    continue;
                }
            };

            let filtered_end_cnt: usize = match end_time_all_index_info
                .iter()
                .find(|item| item.index_name == index_name)
            {
                Some(item) => item.cnt,
                None => {
                    error!(
                        "[ReportServiceImpl->generate_alarm_index_details] Index not found in end_time_all_index_info: {}",
                        index_name
                    );
                    continue;
                }
            };

            let difference: usize = filtered_start_cnt.abs_diff(filtered_end_cnt);
            let difference_percent: f64 = (difference as f64 / filtered_start_cnt as f64) * 100.0;

            let filtered_alarm_cnt: u64 = match alarm_report_infos
                .buckets()
                .iter()
                .find(|item| item.name == index_name)
            {
                Some(item) => item.count,
                None => {
                    warn!(
                        "[ReportServiceImpl->generate_alarm_index_details] Index not found in filtered_alarm_cnt: {}",
                        index_name
                    );
                    0
                }
            };

            let alarm_index_detail: AlarmIndexDetailInfo = AlarmIndexDetailInfo::new(
                index_name.to_string(),
                filtered_start_cnt,
                filtered_end_cnt,
                difference,
                difference_percent,
                filtered_alarm_cnt,
            );

            alarm_index_details.push(alarm_index_detail);
        } //for

        alarm_index_details
    }

    async fn generate_alram_index_diff_details(
        &self,
        index_list: &Vec<IndexConfig>,
        mon_index_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> anyhow::Result<Vec<AlarmIndexDiffDetailInfo>> {
        let mut alarm_index_diff_details: Vec<AlarmIndexDiffDetailInfo> = Vec::new();

        for index in index_list {
            let index_name: &str = index.index_name();

            /* Computes the min and max document counts over a given period. */
            let (min_value, max_value) = self
                .query_service
                .fetch_max_min_doc_count_value(mon_index_name, index_name, start_time, end_time)
                .await?;

            let divisor: f64 = if min_value == 0.0 {
                1.0
            } else {
                min_value
            };

            let diff_cnt: f64 = (max_value - min_value).abs();
            let diff_per: f64 = (diff_cnt / divisor ) * 100.0;

            let alarm_index_diff_info: AlarmIndexDiffDetailInfo = AlarmIndexDiffDetailInfo::new(
                index_name.to_string(),
                max_value as u64,
                min_value as u64,
                diff_cnt as u64,
                diff_per,
            );

            alarm_index_diff_details.push(alarm_index_diff_info);
        }

        Ok(alarm_index_diff_details)
    }

    #[doc = "리포트용 HTML 생성 (템플릿 기반)"]
    fn generate_daily_report_html(
        &self,
        index_list: &IndexListConfig,
        start_local_time: DateTime<Local>,
        end_local_time: DateTime<Local>,
        report_type: ReportType,
        start_time_all_index_cnt: usize,
        end_time_all_index_cnt: usize,
        total_difference: usize,
        alaram_index_cnt: u64,
        total_alarm_cnt: u64,
        alarm_index_details: Vec<AlarmIndexDetailInfo>,
        alarm_index_diff_details: Vec<AlarmIndexDiffDetailInfo>,
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
            ReportType::Day => "일일",
            ReportType::Week => "주간",
            ReportType::Month => "월간",
            ReportType::Year => "연간",
        };

        let agg_interval: String = format!(
            "{} ~ {}",
            convert_data_to_str_human(start_local_time, Local),
            convert_data_to_str_human(end_local_time, Local)
        );

        /* Replacing template placeholders */
        let html_content: String = template_content
            .replace("{{REPORT_TYPE}}", report_name)
            .replace("{{REPORT_INTERVAL}}", &agg_interval)
            .replace(
                "{{REPORT_DATE}}",
                &convert_data_to_str_human(end_local_time, Local),
            )
            .replace(
                "{{TOTAL_INDICES}}",
                &index_list.index().len().to_formatted_string(&Locale::en),
            )
            .replace(
                "{{TOTAL_DOCS_START}}",
                &start_time_all_index_cnt.to_formatted_string(&Locale::en),
            )
            .replace(
                "{{TOTAL_DOCS_END}}",
                &end_time_all_index_cnt.to_formatted_string(&Locale::en),
            )
            .replace(
                "{{TOTAL_CHANGE}}",
                &total_difference.to_formatted_string(&Locale::en),
            )
            .replace("{{CHANGE_STYLE}}", "")
            .replace(
                "{{INDICES_WITH_ALERTS}}",
                &alaram_index_cnt.to_formatted_string(&Locale::en),
            )
            .replace(
                "{{TOTAL_ALERTS}}",
                &total_alarm_cnt.to_formatted_string(&Locale::en),
            )
            .replace(
                "{{INDEX_ROWS}}",
                &self.generate_index_detail_rows(&alarm_index_details),
            )
            .replace(
                "{{INDEX_DIFF_ROWS}}",
                &self.generate_index_diff_detail_rows(&alarm_index_diff_details),
            );

        Ok(html_content)
    }

    #[doc = "인덱스별 상세 정보 테이블 행 생성"]
    fn generate_index_detail_rows(&self, alarm_index_details: &[AlarmIndexDetailInfo]) -> String {
        self.generate_table_rows(alarm_index_details, |alarm_index| {

            format!(
                r#"<tr>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                </tr>"#,
                alarm_index.index_name(),
                alarm_index.start_index_cnt.to_formatted_string(&Locale::en),
                alarm_index.end_index_cnt.to_formatted_string(&Locale::en),
                alarm_index.difference.to_formatted_string(&Locale::en),
                format!("{:.2}%", alarm_index.difference_percent),
                alarm_index.alarm_cnt.to_formatted_string(&Locale::en),
            )
        })
    }

    fn generate_index_diff_detail_rows(
        &self,
        alarm_index_diff_details: &[AlarmIndexDiffDetailInfo],
    ) -> String {
        self.generate_table_rows(alarm_index_diff_details, |alarm_diff_info| {
            format!(
                r#"<tr>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                    <td style="border: 1px solid #ddd; padding: 12px; text-align: left; background-color: #fff;">{}</td>
                </tr>"#,
                alarm_diff_info.index_name(),
                alarm_diff_info.min_index_cnt.to_formatted_string(&Locale::en),
                alarm_diff_info.max_index_cnt.to_formatted_string(&Locale::en),
                alarm_diff_info.difference.to_formatted_string(&Locale::en),
                format!("{:.2}%", alarm_diff_info.difference_percent)
            )
        })
    }

    #[doc = "Helper function for creating common table rows"]
    fn generate_table_rows<T, F>(&self, data: &[T], row_formatter: F) -> String
    where
        F: Fn(&T) -> String,
    {
        data.iter().map(row_formatter).collect::<String>()
    }
}

#[async_trait]
impl<Q, C, N> ReportService for ReportServiceImpl<Q, C, N>
where
    Q: QueryService + Sync,
    C: ChartService,
    N: NotificationService,
{
    #[doc = "Function that provides a report service"]
    async fn report_loop(
        &self,
        mon_index_name: &str,
        alarm_index_name: &str,
        target_index_info_list: &IndexListConfig,
        report_type: ReportType,
    ) -> anyhow::Result<()> {
        let report_config: &ReportConfig = match report_type {
            ReportType::Day => get_daily_report_config_info(),
            ReportType::Week => get_weekly_report_config_info(),
            ReportType::Month => get_monthly_report_config_info(),
            ReportType::Year => get_yearly_report_config_info(),
        };

        if !report_config.enabled {
            info!(
                "[MainController->daily_report_loop] Daily report is disabled. Skipping daily report scheduler."
            );

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        }

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
            /* The reporting schedule is based on Korean time - GMT+9 */
            let now_local: DateTime<Local> = chrono::Local::now();

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

            /* The function runs when it's time to send the report email. */
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
