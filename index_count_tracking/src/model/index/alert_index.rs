use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct AlertIndex {
    pub index_name: String,
    pub cnt: usize,
    pub timestamp: String,
}
