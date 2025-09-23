use crate::common::*;

use crate::model::index::alert_index_format::*;

#[derive(Serialize, Deserialize, Debug, Getters, new)]
#[getset(get = "pub")]
pub struct LogIndexResult {
    pub index_name: String,
    pub alert_yn: bool,
    pub alert_index_format: Option<AlertIndexFormat>,
}
