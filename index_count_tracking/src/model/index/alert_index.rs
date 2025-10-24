use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct AlertIndex {
    pub index_name: String,
    pub cnt: usize,
    // #[serde(default)]
    // pub prev_cnt: usize,
    // pub cur_prev_diff: usize,
    pub timestamp: String,
}
