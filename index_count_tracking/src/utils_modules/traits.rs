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

/* =============== 숫자 변환 트레이트: 정수/부동소수 모두 지원 =============== */
pub trait FromJsonNumeric: Sized {
    fn from_u64(u: u64) -> Option<Self> {
        let _ = u;
        None
    }
    fn from_i64(i: i64) -> Option<Self> {
        let _ = i;
        None
    }
    fn from_f64(f: f64) -> Option<Self> {
        let _ = f;
        None
    }
}

macro_rules! impl_from_json_numeric_int {
    ($($t:ty),* $(,)?) => {
        $(impl FromJsonNumeric for $t {
            fn from_u64(u: u64) -> Option<Self> { <$t>::try_from(u).ok() }
            fn from_i64(i: i64) -> Option<Self> { <$t>::try_from(i).ok() }
        })*
    };
}

impl_from_json_numeric_int!(u64, u32, u16, u8, usize, i64, i32, i16, i8, isize);

impl FromJsonNumeric for f64 {
    fn from_u64(u: u64) -> Option<Self> {
        Some(u as f64)
    }
    fn from_i64(i: i64) -> Option<Self> {
        Some(i as f64)
    }
    fn from_f64(f: f64) -> Option<Self> {
        f.is_finite().then_some(f)
    }
}

impl FromJsonNumeric for f32 {
    fn from_u64(u: u64) -> Option<Self> {
        Some(u as f32)
    }
    fn from_i64(i: i64) -> Option<Self> {
        Some(i as f32)
    }
    fn from_f64(f: f64) -> Option<Self> {
        f.is_finite().then_some(f as f32)
    }
}
