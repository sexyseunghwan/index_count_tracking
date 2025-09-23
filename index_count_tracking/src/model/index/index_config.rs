use crate::common::*;

#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct IndexConfig {
    pub index_name: String,
    pub allowable_fluctuation_range: f64,
    pub agg_term_sec: i64,
}
