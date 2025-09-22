use crate::common::*;

use crate::model::index::index_config::*;

#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct IndexListConfig {
    pub index: Vec<IndexConfig>,
}
