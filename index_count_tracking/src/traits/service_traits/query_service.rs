use crate::common::*;

use crate::model::index::{alert_index::*, index_config::*};

use crate::model::index::alert_index::*;

use crate::dto::log_index_result::*;

// use crate::model::error_alarm_info::*;
// use crate::model::error_alarm_info_format::*;
// use crate::model::vector_index_log_format::*;

#[async_trait]
pub trait QueryService {
    async fn get_index_doc_count(&self, index_name: &str) -> anyhow::Result<usize>;
    async fn post_log_index(
        &self,
        index_name: &str,
        alert_index: &AlertIndex,
    ) -> anyhow::Result<()>;
    //async fn get_max_cnt_from_log_index(&self, index_config: &IndexConfig, cur_timestamp_utc: &str) -> anyhow::Result<AlaramIndexInfo>;
    async fn get_max_cnt_from_log_index(
        &self,
        index_config: &IndexConfig,
        cur_timestamp_utc: &str,
    ) -> anyhow::Result<LogIndexResult>;

    // async fn get_indexing_movement_log(
    //     &self,
    //     query_index: &str,
    //     index_name: &str,
    //     index_type: &str,
    //     start_dt: NaiveDateTime,
    //     end_dt: NaiveDateTime,
    // ) -> Result<VectorIndexLogFormat, anyhow::Error>;
    // async fn post_indexing_error_info(
    //     &self,
    //     index_name: &str,
    //     error_alaram_info: ErrorAlarmInfo,
    // ) -> Result<(), anyhow::Error>;
    // async fn get_error_alarm_infos(
    //     &self,
    //     index_name: &str,
    // ) -> Result<Vec<ErrorAlarmInfoFormat>, anyhow::Error>;
    // async fn delete_index_by_doc(
    //     &self,
    //     index_name: &str,
    //     doc_id: &str,
    // ) -> Result<(), anyhow::Error>;
}
