use chrono_tz::Etc::UTC;

use crate::common::*;

use crate::traits::service_traits::{
    notification_service::*, query_service::*, tracking_monitor_service::*,
};

use crate::model::configs::total_config::*;
use crate::model::index::{alert_index::*, index_list_config::*};

use crate::dto::alarm::alarm_log_history_index::*;
use crate::dto::log_index_result::*;

use crate::utils_modules::time_utils::*;

#[derive(Debug, new)]
pub struct TrackingServiceImpl<Q: QueryService, N: NotificationService> {
    target_query_service: Q,
    mon_query_service: Q,
    notification_service: Arc<N>,
}

impl<Q, N> TrackingServiceImpl<Q, N>
where
    Q: QueryService + Sync + Send,
    N: NotificationService + Sync + Send,
{
    #[doc = "Function that indexes the number if index documents"]
    async fn save_index_cnt_infos(
        &self,
        index_list: &IndexListConfig,
        mon_index_name: &str,
    ) -> anyhow::Result<()> {
        let cur_utc_time: DateTime<Utc> = Utc::now();

        for index_config in index_list.index() {
            let index_name: &str = index_config.index_name();

            /* Number of indexed documents. */
            let doc_cnt: usize = match self
                .target_query_service
                .get_index_doc_count(index_config.index_name())
                .await
            {
                Ok(doc_cnt) => doc_cnt,
                Err(e) => {
                    error!("[TrackingServiceImpl->save_index_cnt_infos]{:?}", e);
                    continue;
                }
            };

            /* Indexes the previous document count to simplify future change-rate cacluations. */
            let prev_doc_cnt: AlertIndex = match self
                .mon_query_service
                .get_latest_index_count_infos(mon_index_name, index_name)
                .await
            {
                Ok(prev_doc_cnt) => prev_doc_cnt,
                Err(e) => {
                    error!("[TrackingServiceImpl->save_index_cnt_infos]{:?}", e);
                    AlertIndex::new(
                        index_name.to_string(),
                        0,
                        0,
                        0,
                        convert_date_to_str(cur_utc_time, UTC),
                    )
                }
            };

            /* Caculate the absolute value */
            let cur_prev_diff: usize = doc_cnt.abs_diff(prev_doc_cnt.cnt);

            let alert_index: AlertIndex = AlertIndex::new(
                index_name.to_string(),
                doc_cnt,
                prev_doc_cnt.cnt,
                cur_prev_diff,
                convert_date_to_str(cur_utc_time, Utc),
            );
            
            self.mon_query_service
                .post_log_index(mon_index_name, &alert_index)
                .await?;
        }

        Ok(())
    }

    #[doc = "Function that detects and returns index informations whose number of documents fluctuated beyond a threshold within a specific period."]
    async fn detect_abnormal_index_changes(
        &self,
        mon_index_name: &str,
        target_index_info_list: &IndexListConfig,
        cur_timestamp_utc: DateTime<Utc>,
    ) -> anyhow::Result<Vec<LogIndexResult>> {
        let mut log_index_results: Vec<LogIndexResult> = Vec::new();

        for index_config in target_index_info_list.index() {
            let log_index_res: LogIndexResult = self
                .mon_query_service
                .get_alert_infos_from_log_index(mon_index_name, index_config, cur_timestamp_utc)
                .await?;

            if log_index_res.alert_yn {
                log_index_results.push(log_index_res);
            }
        }

        Ok(log_index_results)
    }
    
    #[doc = "Function that sends current index information via alerts."]
    async fn alert_index_status(&self, log_index_res: &[LogIndexResult]) -> anyhow::Result<()> {
        info!(
            "Sending index alert for {} problematic indices",
            log_index_res.len()
        );

        /*
            It sends alert via email and Telegram.
        */
        match self
            .notification_service
            .send_index_alert_message(log_index_res)
            .await
        {
            Ok(_) => {
                info!("Successfully sent index alert notifications");
            }
            Err(e) => {
                error!(
                    "[ERROR][MainController->alert_index_status] Failed to send alert notifications: {:?}",
                    e
                );
                return Err(e);
            }
        }

        Ok(())
    }
    
    #[doc = "Function that records alarm history"]
    async fn logging_alarm_history_infos(
        &self,
        index_doc_verification: &Vec<LogIndexResult>,
        cur_timestamp_utc: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let loggin_index_name: &str = get_alarm_log_index_info().index_name();

        for index_result in index_doc_verification {
            let alarm_history_form: AlarmLogHistoryIndex = AlarmLogHistoryIndex::new(
                index_result.index_name().to_string(),
                index_result.cur_cnt,
                index_result.fluctuation_val,
                convert_date_to_str(cur_timestamp_utc, Utc),
            );

            self.mon_query_service
                .post_alarm_history_index(loggin_index_name, alarm_history_form)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<Q, N> TrackingMonitorService for TrackingServiceImpl<Q, N>
where
    Q: QueryService + Sync + Send,
    N: NotificationService + Sync + Send,
{
    #[doc = "Function that continuosly monitors the number of documents in a specific index."]
    async fn tracking_monitor_loop(
        &self,
        mon_index_name: &str,
        target_index_info_list: &IndexListConfig,
    ) -> anyhow::Result<()> {

        /* Schedule ticker */
        let mut ticker: Interval = interval(Duration::from_secs(5));

        loop {
            ticker.tick().await;

            /* 1. Store index document count information. */
            if let Err(e) = self
                .save_index_cnt_infos(target_index_info_list, mon_index_name)
                .await
            {
                error!(
                    "[MainController->monitoring_loop] Failed to save index count infos: {:?}",
                    e
                );
                continue;
            }

            let cur_timestamp_utc: DateTime<Utc> = Utc::now();

            /* 
                2. Verify the number if index documents
                - To send a notification when there is an abnormality in the rate of change of the number of indexes. 
            */
            let index_doc_verification: Vec<LogIndexResult> = match self
                .detect_abnormal_index_changes(mon_index_name, target_index_info_list, cur_timestamp_utc)
                .await
            {
                Ok(results) => results,
                Err(e) => {
                    error!(
                        "[MainController->monitoring_loop] Failed to verify index count: {:?}",
                        e
                    );
                    continue;
                }
            };

            if !index_doc_verification.is_empty() {

                /* 3. Save alarm information to keep alarm history */
                if let Err(e) = self
                    .logging_alarm_history_infos(&index_doc_verification, cur_timestamp_utc)
                    .await
                {
                    error!("[MainController->monitoring_loop] {:?}", e);
                }

                /* 4. It sends an alert based on the verification results. */
                if let Err(e) = self.alert_index_status(&index_doc_verification).await {
                    error!(
                        "[MainController->monitoring_loop] Failed to send alert: {:?}",
                        e
                    );
                }
            }
        }
    }
}
