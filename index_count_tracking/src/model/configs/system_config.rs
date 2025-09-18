use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters)]
#[getset(get = "pub")]
pub struct SystemConfig {
    pub log_index_name: String,
    pub schedule_term: u64,
    pub err_monitor_index: String,
    pub message_chunk_size: usize,
}
