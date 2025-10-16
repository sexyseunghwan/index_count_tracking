#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    OneDay,
    OneWeek,
    OneMonth,
    OneYear,
}

pub fn get_days(report_type: ReportType) -> i64 {
    match report_type {
        ReportType::OneDay => 1,
        ReportType::OneWeek => 7,
        ReportType::OneMonth => 30,
        ReportType::OneYear => 365,
    }
}
