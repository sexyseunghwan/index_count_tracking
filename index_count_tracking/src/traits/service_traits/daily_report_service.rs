use crate::common::*;
use crate::model::{index::index_list_config::*, report::daily_report::*};

#[async_trait]
pub trait DailyReportService {
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
