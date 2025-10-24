use crate::common::*;

#[derive(Debug, Clone, Getters, new)]
#[getset(get = "pub")]
pub struct AlarmIndexDiffDetailInfo {
    pub index_name: String,
    pub max_index_cnt: u64,
    pub min_index_cnt: u64,
    pub difference: u64,
    pub difference_percent: f64,
}
