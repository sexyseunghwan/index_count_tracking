use crate::common::*;
use crate::model::{index::index_list_config::*, report::daily_report::*};

use crate::enums::report_type::*;

#[async_trait]
pub trait ReportService {
    async fn report_loop(
        &self,
        mon_index_name: &str,
        target_index_info_list: &IndexListConfig,
        report_type: ReportType
    ) -> anyhow::Result<()>;
    // async fn generate_daily_report(
    //     &self,
    //     target_index_info_list: &IndexListConfig,
    //     mon_index_name: &str
    // ) -> anyhow::Result<()>;
    // async fn generate_daily_report(
    //     &self,
    //     index_list: &IndexListConfig,
    //     mon_index_name: &str,
    // ) -> anyhow::Result<DailyReport>;
    // async fn send_daily_report_email(&self, report: &DailyReport) -> anyhow::Result<()>;
    // async fn get_index_count_at_time(
    //     &self,
    //     mon_index_name: &str,
    //     index_name: &str,
    //     timestamp: &str,
    // ) -> anyhow::Result<usize>;
    // async fn get_alert_count_in_period(
    //     &self,
    //     mon_index_name: &str,
    //     index_name: &str,
    //     start_time: &str,
    //     end_time: &str,
    // ) -> anyhow::Result<usize>;
}
