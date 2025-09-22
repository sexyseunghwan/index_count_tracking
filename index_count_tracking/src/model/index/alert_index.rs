use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters, new)]
#[getset(get = "pub")]
pub struct AlertIndex {
    pub index_name: String,
    pub cnt: usize,
    pub timestamp: String,
}
