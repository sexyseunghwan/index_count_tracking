use crate::common::*;

use crate::utils_modules::{io_utils::*, time_utils::*};

use crate::model::{
    index::{alert_index::*, index_list_config::*},
    configs::total_config::*
};

use crate::env_configuration::env_config::*;

use crate::traits::service_traits::{notification_service::*, query_service::*};

#[derive(Debug, new)]
pub struct MainController<N: NotificationService, TQ: QueryService, MQ: QueryService> {
    notification_service: N,
    target_query_service: TQ,
    mon_query_service: MQ,
}

impl<N: NotificationService, TQ: QueryService, MQ: QueryService> MainController<N, TQ, MQ> {
    #[doc = ""]
    pub async fn main_task(&self) -> anyhow::Result<()> {
        
        let index_list: IndexListConfig = read_toml_from_file::<IndexListConfig>(&INDEX_LIST_PATH)?;    
        let mon_index_name: &str = get_system_config_info().monitor_index_name();
        
        /* 인덱스 문서 개수 정보 저장 */
        self.save_index_cnt_infos(&index_list, mon_index_name).await?;
        
        /* 인덱스 문서 개수 검증 */
        
        // /* 인덱스 문서 개수 검증 */
        // for index_config in index_list.index() {
        //     let index_name: &str = index_config.index_name();


        // }

        // let mut ticker: Interval = interval(Duration::from_secs(10));

        // loop {

        //     ticker.tick().await;

        //     /* 1. 특정 인덱스 문서 개수를 카운트 -> 10초에 한번씩? */
        //     for index_config in index_list.index() {

        //         let test: usize =
        //             self.target_query_service.get_index_doc_count(index_config.index_name()).await?;

        //         println!("{} -> {}", index_config.index_name(), test);

        //     }

        //     /* 2. 집계를 하여 문제가 있는 경우에는 알람을 보냄 */
        // }

        Ok(())
    }
    
    #[doc = "인덱스 문서 개수 정보 색인 해주는 함수"]
    async fn save_index_cnt_infos(&self, index_list: &IndexListConfig, mon_index_name: &str) -> anyhow::Result<()> {

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
                AlertIndex::new(index_name.to_string(), doc_cnt, cur_timestamp_utc.clone());
            
            /* 해당 정보를 모니터링 클러스터에 색인 */
            self.mon_query_service.post_log_index(mon_index_name, &alert_index).await?;
        }
        
        Ok(())
    }
    
    #[doc = "인덱스 문서 개수 검증"]
    async fn verify_index_cnt(&self, index_list: &IndexListConfig, mon_index_name: &str) -> anyhow::Result<()> {

        let cur_timestamp_utc: String = get_current_utc_naivedatetime_str();
        
        for index_config in index_list.index() {
            //let index_name: &str = index_config.index_name();
            
            
            
        }

        Ok(())
    }

}
