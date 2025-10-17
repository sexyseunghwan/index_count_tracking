use crate::common::*;

use crate::utils_modules::traits::*;

#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct IndexNameCount {
    pub name: String,
    pub count: u64,
}

impl FromAggBucket for IndexNameCount {
    fn from_agg_bucket(bucket: &Value) -> anyhow::Result<Self> {
        let name: String = bucket
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow!("[IndexNameCount->from_agg_bucket] bucket.key is missing or not a string")
            })?
            .to_string();

        let count: u64 = bucket
            .get("doc_count")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                anyhow!(
                    "[IndexNameCount->from_agg_bucket] bucket.doc_count is missing or not a u64"
                )
            })?;

        Ok(IndexNameCount { name, count })
    }
}
