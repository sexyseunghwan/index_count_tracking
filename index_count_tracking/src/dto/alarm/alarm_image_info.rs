use crate::common::*;

#[derive(Debug, Clone, Getters, new)]
#[getset(get = "pub")]
pub struct AlarmImageInfo {
    pub index_name: String,
    pub pic_path: PathBuf,
}
