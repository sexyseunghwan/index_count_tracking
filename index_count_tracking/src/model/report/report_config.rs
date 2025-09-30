use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub enabled: bool,
    pub cron_schedule: String,
    pub timezone: String,
    pub subject_prefix: String,
    pub include_charts: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cron_schedule: "0 0 9 * * *".to_string(), // 매일 9시
            timezone: "UTC".to_string(),
            subject_prefix: "[인덱스 모니터링]".to_string(),
            include_charts: false,
        }
    }
}