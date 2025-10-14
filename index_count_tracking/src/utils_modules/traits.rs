use crate::common::*;

/* Elasticsearch hit → 도메인 타입 변환을 위한 공통 트레이트 */
pub trait FromSearchHit<S>
where
    S: DeserializeOwned,
{
    fn from_search_hit(doc_id: String, source: S) -> Self;
}

/* Elasticsearch aggregation bucket → 도메인 타입 변환을 위한 공통 트레이트 */
pub trait FromAggBucket
where
    Self: Sized,
{
    fn from_agg_bucket(bucket: &Value) -> Result<Self, anyhow::Error>;
}
