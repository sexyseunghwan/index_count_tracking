use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReport {
    pub date: String,
    pub summary: ReportSummary,
    pub index_stats: Vec<IndexDailyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_indices: usize,
    pub total_documents_start: usize,
    pub total_documents_end: usize,
    pub total_change: i64,
    pub indices_with_alerts: usize,
    pub total_alerts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDailyStats {
    pub index_name: String,
    pub start_count: usize,
    pub end_count: usize,
    pub change: i64,
    pub change_percentage: f64,
    pub alert_count: usize,
    pub status: IndexStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexStatus {
    Normal,
    Warning,
    Critical,
}

impl ReportSummary {
    pub fn calculate(index_stats: &[IndexDailyStats]) -> Self {
        let total_indices = index_stats.len();
        let total_documents_start = index_stats.iter().map(|s| s.start_count).sum();
        let total_documents_end = index_stats.iter().map(|s| s.end_count).sum();
        let total_change = index_stats.iter().map(|s| s.change).sum();
        let indices_with_alerts = index_stats.iter().filter(|s| s.alert_count > 0).count();
        let total_alerts = index_stats.iter().map(|s| s.alert_count).sum();

        Self {
            total_indices,
            total_documents_start,
            total_documents_end,
            total_change,
            indices_with_alerts,
            total_alerts,
        }
    }
}

impl IndexDailyStats {
    pub fn new(
        index_name: String,
        start_count: usize,
        end_count: usize,
        alert_count: usize,
    ) -> Self {
        let change = end_count as i64 - start_count as i64;
        let change_percentage = if start_count > 0 {
            (change as f64 / start_count as f64) * 100.0
        } else {
            0.0
        };

        let status = if alert_count > 10 {
            IndexStatus::Critical
        } else if alert_count > 0 {
            IndexStatus::Warning
        } else {
            IndexStatus::Normal
        };

        Self {
            index_name,
            start_count,
            end_count,
            change,
            change_percentage,
            alert_count,
            status,
        }
    }
}