use crate::common::*;

use crate::enums::index_status::*;

#[derive(Debug, Clone, Getters, new)]
#[getset(get = "pub")]
pub struct AlarmIndexDetailInfo {
    pub index_name: String,
    pub start_index_cnt: usize,
    pub end_index_cnt: usize,
    pub difference: usize,
    pub difference_percent: usize,
    pub alarm_cnt: u64
}
