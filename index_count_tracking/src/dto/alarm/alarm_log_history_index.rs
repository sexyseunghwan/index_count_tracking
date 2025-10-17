use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, new)]
pub struct AlarmLogHistoryIndex {
    pub index_name: String,
    pub index_cnt: usize,
    pub fluctuation_val: f64,
    pub timestamp: String,
}
