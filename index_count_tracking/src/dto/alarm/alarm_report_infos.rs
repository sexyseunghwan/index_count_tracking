use crate::common::*;

use crate::dto::index_name_count::*;

#[derive(Debug, Clone, Serialize, Deserialize, Getters, new)]
#[getset(get = "pub")]
pub struct AlarmReportInfos {
    pub buckets: Vec<IndexNameCount>,
    pub distinct_count_u64: u64,
}
