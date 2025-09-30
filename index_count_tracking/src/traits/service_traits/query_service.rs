use crate::common::*;

use crate::model::index::{alert_index::*, index_config::*};


use crate::dto::log_index_result::*;

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
        cur_timestamp_utc: &str,
    ) -> anyhow::Result<LogIndexResult>;

    async fn execute_search_query(
        &self,
        index_name: &str,
        query: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value>;
}
