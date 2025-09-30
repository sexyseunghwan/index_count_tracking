use crate::common::*;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Clone, Serialize)]
pub struct SortSpec<'a> {
    pub field: &'a str,
    pub order: SortOrder,
}

impl<'a> SortSpec<'a> {
    pub fn to_es_json(&self) -> serde_json::Value {
        serde_json::json!({ self.field: { "order": self.order } })
    }
}
