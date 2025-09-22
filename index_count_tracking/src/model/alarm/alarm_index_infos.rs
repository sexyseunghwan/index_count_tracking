use crate::common::*;

#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct AlaramIndexInfo {
    pub index_name: String,
    pub doc_cnt: usize,
    pub alert_yn: bool
}