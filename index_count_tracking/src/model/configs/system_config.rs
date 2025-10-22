use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters)]
#[getset(get = "pub")]
pub struct SystemConfig {
    pub monitor_index_name: String,
    pub message_chunk_size: usize,
    pub ticker_sec: u64
}
