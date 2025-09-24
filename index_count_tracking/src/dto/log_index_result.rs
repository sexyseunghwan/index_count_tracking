use crate::common::*;

use crate::model::index::alert_index::*;

#[derive(Serialize, Deserialize, Debug, Getters, Setters, new)]
#[getset(get = "pub",set = "pub")]
pub struct LogIndexResult {
    pub index_name: String,
    pub alert_yn: bool,
    pub alert_index_format: Option<Vec<AlertIndex>>,
}
