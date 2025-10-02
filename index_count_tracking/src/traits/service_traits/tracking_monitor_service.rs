use crate::common::*;

use crate::model::index::index_list_config::*;

#[async_trait]
pub trait TrackingMonitorService {
    async fn tracking_monitor_loop(
        &self,
        mon_index_name: &str,
        target_index_info_list: &IndexListConfig,
    ) -> anyhow::Result<()>;
}
