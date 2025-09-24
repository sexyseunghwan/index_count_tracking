use crate::common::*;

use crate::utils_modules::{io_utils::*, time_utils::*};

use crate::model::{
    configs::total_config::*,
    index::{alert_index::*, index_list_config::*},
};

use crate::dto::log_index_result::*;

use crate::env_configuration::env_config::*;

use crate::traits::service_traits::{notification_service::*, query_service::*};

#[derive(Debug, new)]
pub struct MainController<N: NotificationService, TQ: QueryService, MQ: QueryService> {
    notification_service: N,
    target_query_service: TQ,
    mon_query_service: MQ,
}

impl<N: NotificationService, TQ: QueryService, MQ: QueryService> MainController<N, TQ, MQ> {
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
        let index_list: IndexListConfig = read_toml_from_file::<IndexListConfig>(&INDEX_LIST_PATH)?;
        let mon_index_name: &str = get_system_config_info().monitor_index_name();

        let mut ticker: Interval = interval(Duration::from_secs(30));

        loop {

            ticker.tick().await;

            /* 1. 인덱스 문서 개수 정보 저장 */
            self.save_index_cnt_infos(&index_list, mon_index_name)
                .await?;

            /* 2. 인덱스 문서 개수 검증 */
            let index_doc_verification: Vec<LogIndexResult> =
                self.verify_index_cnt(mon_index_name, &index_list).await?;

            /* 3. 검증 결과를 바탕으로 알람을 보내주는 로직 */
            self.alert_index_status(&index_doc_verification).await?;

        }
    }
    
    #[doc = "인덱스 문서 개수 정보 색인 해주는 함수"]
    async fn save_index_cnt_infos(
        &self,
        index_list: &IndexListConfig,
        mon_index_name: &str,
    ) -> anyhow::Result<()> {
        let cur_timestamp_utc: String = get_current_utc_naivedatetime_str();

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
            let alert_index: AlertIndex =
                AlertIndex::new(index_name.to_string(), doc_cnt, cur_timestamp_utc.to_string());

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
        index_list: &IndexListConfig,
    ) -> anyhow::Result<Vec<LogIndexResult>> {
        let cur_timestamp_utc: String = get_current_utc_naivedatetime_str();

        let mut log_index_results: Vec<LogIndexResult> = Vec::new();

        for index_config in index_list.index() {
            let log_index_res: LogIndexResult = self
                .mon_query_service
                .get_alert_infos_from_log_index(mon_index_name, index_config, &cur_timestamp_utc)
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
}