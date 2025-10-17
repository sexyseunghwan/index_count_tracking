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
    #[doc = "인덱스 문서 개수 정보 색인 해주는 함수"]
    async fn save_index_cnt_infos(
        &self,
        index_list: &IndexListConfig,
        mon_index_name: &str,
    ) -> anyhow::Result<()> {
        let cur_utc_time: DateTime<Utc> = Utc::now();

        for index_config in index_list.index() {
            let index_name: &str = index_config.index_name();

            /* 해당 인덱스의 문서 개수 */
            let doc_cnt: usize = match self
                .target_query_service
                .get_index_doc_count(index_config.index_name())
                .await
            {
                Ok(doc_cnt) => doc_cnt,
                Err(e) => {
                    error!("{:?}", e);
                    continue;
                }
            };

            /* 모니터링 인덱스에 해당 인덱스의 문서수를 색인 */
            let alert_index: AlertIndex = AlertIndex::new(
                index_name.to_string(),
                doc_cnt,
                convert_date_to_str(cur_utc_time, Utc),
            );

            /* 해당 정보를 모니터링 클러스터에 색인 */
            self.mon_query_service
                .post_log_index(mon_index_name, &alert_index)
                .await?;
        }

        Ok(())
    }

    #[doc = "인덱스 문서 개수 검증"]
    async fn verify_index_cnt(
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

    #[doc = "인덱스 문서 개수 이상 상황 알람 발송"]
    async fn alert_index_status(&self, log_index_res: &[LogIndexResult]) -> anyhow::Result<()> {
        info!(
            "Sending index alert for {} problematic indices",
            log_index_res.len()
        );

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

    #[doc = "알람 이력을 기록해주는 함수"]
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
    #[doc = "특정 인덱스의 문서 개수를 계속해서 모니터링 해주는 함수"]
    async fn tracking_monitor_loop(
        &self,
        mon_index_name: &str,
        target_index_info_list: &IndexListConfig,
    ) -> anyhow::Result<()> {
        /* 30초에 한번씩 스케쥴 */
        let mut ticker: Interval = interval(Duration::from_secs(5));

        loop {
            ticker.tick().await;

            /* 1. 인덱스 문서 개수 정보 저장 */
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

            /* 2. 인덱스 문서 개수 검증 */
            let index_doc_verification: Vec<LogIndexResult> = match self
                .verify_index_cnt(mon_index_name, target_index_info_list, cur_timestamp_utc)
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

            if index_doc_verification.len() > 0 {
                /* 3. 알람 히스토리 보관을 위해서 로깅해주는 로직 */
                if let Err(e) = self
                    .logging_alarm_history_infos(&index_doc_verification, cur_timestamp_utc)
                    .await
                {
                    error!("[MainController->monitoring_loop] {:?}", e);
                }

                /* 4. 검증 결과를 바탕으로 알람을 보내주는 로직 */
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
