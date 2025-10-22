use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub enabled: bool,
    pub cron_schedule: String
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cron_schedule: "0 0 9 * * *".to_string(),
        }
    }
}
