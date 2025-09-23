use crate::common::*;

use crate::model::index::alert_index::*;

use crate::utils_modules::traits::*;

#[derive(Serialize, Deserialize, Debug, Getters, new)]
#[getset(get = "pub")]
pub struct AlertIndexFormat {
    pub doc_id: String,
    pub alert_index: AlertIndex,
}

impl FromSearchHit<AlertIndex> for AlertIndexFormat {
    fn from_search_hit(doc_id: String, vector_index_log: AlertIndex) -> Self {
        AlertIndexFormat::new(doc_id, vector_index_log)
    }
}
