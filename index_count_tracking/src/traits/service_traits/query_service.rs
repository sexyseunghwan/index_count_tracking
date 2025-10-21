use crate::common::*;

use crate::model::index::{alert_index::*, index_config::*};

use crate::dto::{
    alarm::{alarm_log_history_index::*, alarm_report_infos::*},
    index_count_agg_result::*,
    index_name_count::*,
    log_index_result::*,
};

#[async_trait]
pub trait QueryService {
    async fn get_index_doc_count(&self, index_name: &str) -> anyhow::Result<usize>;
    async fn post_log_index(
        &self,
        index_name: &str,
        alert_index: &AlertIndex,
    ) -> anyhow::Result<()>;
    async fn get_alert_infos_from_log_index(
        &self,
        mon_index_name: &str,
        index_config: &IndexConfig,
        cur_timestamp_utc: DateTime<Utc>,
    ) -> anyhow::Result<LogIndexResult>;
    async fn get_report_infos_from_log_index(
        &self,
        mon_index_name: &str,
        index_name: &str,
        start_timestamp: DateTime<Utc>,
        end_timestamp: DateTime<Utc>,
    ) -> anyhow::Result<Vec<AlertIndex>>;
    async fn post_alarm_history_index(
        &self,
        index_name: &str,
        alarm_history_index: AlarmLogHistoryIndex,
    ) -> anyhow::Result<()>;
    async fn get_start_time_all_indicies_count(
        &self,
        mon_index_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> anyhow::Result<Vec<IndexCountAggResult>>;
    async fn get_end_time_all_indicies_count(
        &self,
        mon_index_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> anyhow::Result<Vec<IndexCountAggResult>>;
    async fn get_index_name_aggregations(
        &self,
        alarm_index_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> anyhow::Result<AlarmReportInfos>;
    async fn get_latest_index_count_infos(
        &self,
        mon_index_name: &str,
        param_index_name: &str,
    ) -> anyhow::Result<AlertIndex>;
    async fn fetch_max_doc_count_variation(
        &self,
        mon_index_name: &str,
        index_name: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> anyhow::Result<AlertIndex>;
}
