use crate::common::*;

#[derive(Debug, Clone, Getters, new)]
#[getset(get = "pub")]
pub struct AlarmIndexDiffDetailInfo {
    pub index_name: String,
    pub start_index_cnt: usize,
    pub end_index_cnt: usize,
    pub difference: usize,
    pub difference_percent: usize,
    pub timestamp: String,
}
