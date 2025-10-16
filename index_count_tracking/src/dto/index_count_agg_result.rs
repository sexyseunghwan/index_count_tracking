use crate::common::*;

use crate::utils_modules::traits::*;

#[doc = r#"
    Elasticsearch 집계 응답에서 인덱스별 최초 카운트 정보를 담는 DTO

    # Fields
    * `index_name` - 인덱스명 (bucket의 key)
    * `doc_count` - 버킷 내 문서 수
    * `cnt` - 시간순 시점의 cnt 값
    * `timestamp` - 시간순 시점의 timestamp
"#]
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct IndexCountAggResult {
    pub index_name: String,
    pub doc_count: usize,
    pub cnt: usize,
    pub timestamp: String,
}

impl FromAggBucket for IndexCountAggResult {
    fn from_agg_bucket(bucket: &Value) -> Result<Self, anyhow::Error> {
        /* 1) index_name (bucket의 key) */
        let index_name: String = bucket
            .get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow!("[IndexCountAggResult->from_agg_bucket] Missing or invalid 'key'")
            })?
            .to_string();

        /* 2) doc_count */
        let doc_count: usize = bucket
            .get("doc_count")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                anyhow!("[IndexCountAggResult->from_agg_bucket] Missing or invalid 'doc_count'")
            })?
            .try_into()?;

        /* 3) earliest_cnt.top[0].metrics */
        let metrics: &Value = bucket
            .get("est_cnt")
            .and_then(|ec| ec.get("top"))
            .and_then(|top_arr| top_arr.get(0))
            .and_then(|first| first.get("metrics"))
            .ok_or_else(|| {
                anyhow!("[IndexCountAggResult->from_agg_bucket] Missing 'est_cnt.top[0].metrics'")
            })?;

        /* 4) cnt 값 */
        let earliest_cnt: usize = metrics
            .get("cnt")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                anyhow!("[IndexCountAggResult->from_agg_bucket] Missing or invalid 'cnt'")
            })?
            .try_into()?;

        /* 5) timestamp 값 */
        let earliest_timestamp: String = metrics
            .get("timestamp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow!("[IndexCountAggResult->from_agg_bucket] Missing or invalid 'timestamp'")
            })?
            .to_string();

        Ok(IndexCountAggResult::new(
            index_name,
            doc_count,
            earliest_cnt,
            earliest_timestamp,
        ))
    }
}
