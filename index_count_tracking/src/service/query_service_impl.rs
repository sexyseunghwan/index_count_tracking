use crate::common::*;

use crate::traits::{repository_traits::es_repository::*, service_traits::query_service::*};

use crate::repository::es_repository_impl::*;

use crate::utils_modules::{time_utils::*, traits::*};

use crate::model::alarm::alarm_log_history_index::*;
use crate::model::index::{alert_index::*, alert_index_format::*, index_config::*};

use crate::dto::log_index_result::*;

use crate::enums::sort_order::*;

#[derive(Debug, new)]
pub struct QueryServiceImpl {
    es_conn: Arc<EsRepositoryImpl>,
}

impl QueryServiceImpl {
    #[doc = r#"
        Elasticsearch 검색 응답을 파싱하여 벡터 형태의 구조화된 객체로 변환하는 제네릭 함수.

        1. ES 응답의 `hits.hits` 배열에서 각 검색 결과를 추출
        2. 각 히트의 `_id`와 `_source`를 분리하여 파싱
        3. `_source`를 지정된 타입 `S`로 역직렬화
        4. `FromSearchHit` 트레이트를 통해 최종 타입 `T`로 변환
        5. 모든 결과를 벡터로 수집하여 반환

        # Type Parameters
        * `T` - 최종 반환할 객체 타입 (`FromSearchHit` 트레이트 구현 필요)
        * `S` - ES `_source` 필드의 역직렬화 타입 (`DeserializeOwned` 구현 필요)

        # Arguments
        * `response_body` - Elasticsearch 검색 응답 JSON

        # Returns
        * `Vec<T>` - 변환된 객체들의 벡터
        * `anyhow::Error` - 응답 파싱 실패, 필수 필드 누락, 역직렬화 실패 시
    "#]
    fn get_query_result_vec<T, S>(&self, response_body: &Value) -> Result<Vec<T>, anyhow::Error>
    where
        S: DeserializeOwned,
        T: FromSearchHit<S>,
    {
        let hits: &Value = response_body
            .get("hits")
            .and_then(|h| h.get("hits"))
            .ok_or_else(|| {
                anyhow!("[QueryServiceImpl->get_query_result_vec] Missing 'hits.hits' field")
            })?;

        let arr: &Vec<Value> = hits.as_array().ok_or_else(|| {
            anyhow!("[QueryServiceImpl->get_query_result_vec] 'hits.hits' is not an array")
        })?;

        /* ID + source 역직렬화 → T 로 변환 */
        let results: Vec<T> = arr
            .iter()
            .map(|hit| {
                /* 1) doc_id */
                let id: String = hit
                    .get("_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow!("[QueryServiceImpl->get_query_result_vec] Missing or invalid '_id'")
                    })?
                    .to_string();

                /* 2) source 역직렬화 */
                let src_val: &Value = hit.get("_source").ok_or_else(|| {
                    anyhow!("[QueryServiceImpl->get_query_result_vec] Missing '_source'")
                })?;

                let source: S = serde_json::from_value(src_val.to_owned()).map_err(|e| {
                    anyhow!(
                        "[QueryServiceImpl->get_query_result_vec] Failed to deserialize source: {}",
                        e
                    )
                })?;

                /* 3) 트레이트 메서드로 T 생성 */
                Ok::<T, anyhow::Error>(T::from_search_hit(id, source))
            })
            .collect::<Result<_, _>>()?;
        Ok(results)
    }

    #[doc = r#"
        Elasticsearch 검색 응답에서 첫 번째 결과만을 파싱하여 단일 구조화된 객체로 변환하는 제네릭 함수.

        1. ES 응답의 `hits.hits` 배열에서 첫 번째 검색 결과를 추출
        2. 첫 번째 히트가 없으면 에러 반환 (빈 결과 처리)
        3. `_id`와 `_source`를 분리하여 파싱
        4. `_source`를 지정된 타입 `S`로 역직렬화
        5. `FromSearchHit` 트레이트를 통해 최종 타입 `T`로 변환

        # Type Parameters
        * `T` - 최종 반환할 객체 타입 (`FromSearchHit` 트레이트 구현 필요)
        * `S` - ES `_source` 필드의 역직렬화 타입 (`DeserializeOwned` 구현 필요)

        # Arguments
        * `response_body` - Elasticsearch 검색 응답 JSON

        # Returns
        * `T` - 변환된 단일 객체
        * `anyhow::Error` - 응답 파싱 실패, 빈 결과, 필수 필드 누락, 역직렬화 실패 시
    "#]
    #[allow(dead_code)]
    fn get_query_result<T, S>(&self, response_body: &Value) -> Result<T, anyhow::Error>
    where
        S: DeserializeOwned,
        T: FromSearchHit<S>,
    {
        let hits: &Value = response_body
            .get("hits")
            .and_then(|h| h.get("hits"))
            .ok_or_else(|| {
                anyhow!("[QueryServiceImpl->get_query_result] Missing 'hits.hits' field")
            })?;

        let arr: &Vec<Value> = hits.as_array().ok_or_else(|| {
            anyhow!("[QueryServiceImpl->get_query_result] 'hits.hits' is not an array")
        })?;

        let first_hit: &Value = arr
            .first()
            .ok_or_else(|| anyhow!("[QueryServiceImpl->get_query_result] hits array is empty"))?;

        let id: String = first_hit
            .get("_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow!("[QueryServiceImpl->get_query_result] Missing or invalid '_id'")
            })?
            .to_string();

        let src_val: &Value = first_hit
            .get("_source")
            .ok_or_else(|| anyhow!("[QueryServiceImpl->get_query_result] Missing '_source'"))?;

        let source: S = serde_json::from_value(src_val.to_owned()).map_err(|e| {
            anyhow!(
                "[QueryServiceImpl->get_query_result] Failed to deserialize source: {}",
                e
            )
        })?;

        Ok(T::from_search_hit(id, source))
    }

    #[doc = r#"
        지정된 시간 범위와 인덱스에 대해 Elasticsearch 집계 쿼리를 실행하여 단일 집계 값을 조회하는 헬퍼 함수.

        1. 시간 범위(`timestamp` 필드) 및 인덱스명(`index_name.keyword`)으로 필터링된 bool 쿼리 생성
        2. 지정된 집계 타입(`agg_body`)으로 집계 수행 (예: max, min, avg 등)
        3. 집계 결과에서 `value` 필드를 f64 타입으로 추출
        4. NaN이나 무한대 값 방어를 통해 유효한 값만 반환
        5. 결과가 없거나 유효하지 않으면 None 반환

        # Arguments
        * `mon_index_name` - 모니터링 데이터가 저장된 Elasticsearch 인덱스명
        * `index_name` - 필터링할 대상 인덱스명 (term 쿼리용)
        * `gte` - 시간 범위 시작점 (greater than or equal, ISO 8601 포맷)
        * `lte` - 시간 범위 종료점 (less than or equal, ISO 8601 포맷)
        * `agg_name` - 집계 쿼리의 이름 (응답에서 참조용)
        * `agg_body` - 집계 쿼리 본문 (JSON 형태의 집계 정의)

        # Returns
        * `Option<f64>` - 집계 결과 값 (유효한 경우), None (결과 없음 또는 무효한 값)
        * `anyhow::Error` - ES 조회 실패 시
    "#]
    async fn fetch_agg_value_f64(
        &self,
        mon_index_name: &str,
        index_name: &str,
        gte: DateTime<Utc>,
        lte: DateTime<Utc>,
        agg_name: &str,
        agg_body: Value,
    ) -> anyhow::Result<Option<f64>> {
        let query: Value = json!({
            "size": 0,
            "query": {
                "bool": {
                    "filter": [
                        {
                            "range": {
                                "timestamp": { "gte": convert_date_to_str(gte, Utc), "lte": convert_date_to_str(lte, Utc) }
                            }
                        },
                        {
                            "term": {
                                "index_name.keyword": index_name
                            }
                        }
                    ]
                }

            },
            "aggs": {
                agg_name: agg_body
            }
        });

        let resp: Value = self
            .es_conn
            .get_search_query(&query, mon_index_name)
            .await?;
        let v: Option<f64> = resp["aggregations"][agg_name]["value"].as_f64();

        /* NaN/무한대 방어 */
        let v: Option<f64> = v.and_then(|x| if x.is_finite() { Some(x) } else { None });
        Ok(v)
    }

    #[doc = r#"
        지정된 시간 범위와 인덱스에 대해 cnt 필드의 최대/최소 값을 병렬로 조회한다.

        1. `fetch_agg_value_f64`를 통해 max 집계와 min 집계를 각각 수행
        2. Elasticsearch의 집계 기능을 사용하여 지정된 시간 범위 내에서:
           - `cnt` 필드의 최대값을 조회
           - `cnt` 필드의 최소값을 조회
        3. 각 값이 null이거나 유효하지 않은 경우 0.0으로 기본값 설정

        # Arguments
        * `mon_index_name` - 모니터링 데이터가 저장된 인덱스명
        * `index_name` - 조회 대상 인덱스명 (필터링용)
        * `gte` - 시작 시간 (greater than or equal)
        * `lte` - 종료 시간 (less than or equal)

        # Returns
        * `(f64, f64)` - (최소값, 최대값) 튜플
        * `anyhow::Error` - ES 조회 실패 시
    "#]
    async fn fetch_min_max_values(
        &self,
        mon_index_name: &str,
        index_name: &str,
        gte: DateTime<Utc>,
        lte: DateTime<Utc>,
    ) -> anyhow::Result<(f64, f64)> {
        let max_val: f64 = self
            .fetch_agg_value_f64(
                mon_index_name,
                index_name,
                gte,
                lte,
                "max_value_in_range",
                json!({ "max": { "field": "cnt" } }),
            )
            .await?
            .unwrap_or(0.0);

        let min_val: f64 = self
            .fetch_agg_value_f64(
                mon_index_name,
                index_name,
                gte,
                lte,
                "min_value_in_range",
                json!({ "min": { "field": "cnt" } }),
            )
            .await?
            .unwrap_or(0.0);

        Ok((min_val, max_val))
    }

    #[doc = r#"
        최소값과 최대값을 바탕으로 변동률(%)을 계산하는 헬퍼 함수.

        변동률 = ((최대값 - 최소값) / 최소값) × 100

        1. 최소값이 0보다 큰 경우: 정상적인 변동률 계산 수행  
        2. 최소값이 0 이하인 경우: 0.0 반환 (0으로 나누기 방지)  
        3. 결과는 백분율로 표현됨 (예: 50.0 = 50%)

        # Arguments
        * `min_val` - 최소값
        * `max_val` - 최대값

        # Returns
        * `f64` - 변동률 (백분율)
    "#]
    fn calculate_fluctuation(val1: f64, val2: f64) -> f64 {
        let diff: f64 = (val1 - val2).abs();

        let avg: f64 = (val1.abs() + val2.abs()) / 2.0;

        if avg > 0.0 { (diff / avg) * 100.0 } else { 0.0 }
    }

    // #[doc = r#"
    //     지정된 시간 범위 내에서 특정 인덱스의 알람 데이터를 조회하여 AlertIndexFormat 벡터로 반환한다.

    //     1. 시간 범위(`gte` ~ `lte`)와 인덱스명으로 필터링된 검색 쿼리 생성
    //     2. `timestamp` 필드 기준으로 내림차순 정렬하여 최신 데이터부터 조회
    //     3. 조회된 결과를 `AlertIndex`에서 `AlertIndexFormat`으로 변환
    //     4. `get_query_result_vec` 헬퍼를 통해 ES 응답을 구조화된 객체로 파싱

    //     # Arguments
    //     * `mon_index_name` - 모니터링 데이터가 저장된 인덱스명
    //     * `index_name` - 대상 인덱스명
    //     * `prev_timestamp_utc` - 조회 시작 시간 (UTC)
    //     * `cur_timestamp_utc` - 조회 종료 시간 (UTC)

    //     # Returns
    //     * `Vec<AlertIndexFormat>` - 시간순으로 정렬된 알람 데이터 목록
    //     * `anyhow::Error` - ES 조회 실패 또는 파싱 실패 시
    // "#]

    #[doc = ""]
    async fn fetch_index_cnt_infos<'a>(
        &self,
        mon_index_name: &str,
        index_name: &str,
        prev_timestamp_utc: DateTime<Utc>,
        cur_timestamp_utc: DateTime<Utc>,
        size: usize,
        sorts: Option<Vec<SortSpec<'a>>>,
    ) -> anyhow::Result<Vec<AlertIndexFormat>> {
        let sort_json: Vec<Value> = sorts
            .unwrap_or_default()
            .iter()
            .map(|spec| spec.to_es_json())
            .collect();

        let search_query: Value = json!({
            "query": {
                "bool": {
                    "filter": [
                        {
                            "range": {
                                "timestamp": { "gte": convert_date_to_str(prev_timestamp_utc, Utc), "lte": convert_date_to_str(cur_timestamp_utc, Utc) }
                            }
                        },
                        {
                            "term": {
                                "index_name.keyword": index_name
                            }
                        }
                    ]
                }
            },
            "sort": sort_json,
            "size": size
        });

        let response_body: Value = self
            .es_conn
            .get_search_query(&search_query, mon_index_name)
            .await?;

        self.get_query_result_vec::<AlertIndexFormat, AlertIndex>(&response_body)
    }
}

#[async_trait]
impl QueryService for QueryServiceImpl {
    #[doc = r#"
        지정된 Elasticsearch 인덱스의 전체 문서 개수를 정확하게 조회하여 반환하는 함수.

        1. `match_all` 쿼리로 모든 문서를 대상으로 검색 수행
        2. `size: 0`으로 설정하여 문서 본문은 받지 않고 메타데이터만 조회
        3. `track_total_hits: true`로 설정하여 정확한 총 문서 수 계산 활성화
        4. 응답의 `hits.total.value`에서 문서 개수를 usize로 변환
        5. 큰 인덱스에서도 정확한 카운트를 보장

        # Arguments
        * `index_name` - 문서 개수를 조회할 Elasticsearch 인덱스명

        # Returns
        * `usize` - 인덱스의 총 문서 개수
        * `anyhow::Error` - ES 조회 실패, 응답 파싱 실패, 타입 변환 실패 시
    "#]
    async fn get_index_doc_count(&self, index_name: &str) -> anyhow::Result<usize> {
        let query: Value = json!({
            "size": 0,                     /* 문서 본문은 받지 않음 */
            "track_total_hits": true,      /* 정확한 총건수 계산 */
            "query": { "match_all": {} }
        });

        let resp: Value = self.es_conn.get_search_query(&query, index_name).await?;

        let hits_total: &Value = &resp["hits"]["total"];

        let value: usize = hits_total["value"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("[QueryServiceImpl->get_index_doc_count] invalid hits.total.value in search response"))?
            .try_into()?;

        Ok(value)
    }

    #[doc = r#"
        AlertIndex 구조체를 지정된 Elasticsearch 인덱스에 문서로 색인(저장)하는 함수.

        1. `AlertIndex` 구조체를 JSON 형태로 직렬화
        2. Elasticsearch의 인덱싱 API를 통해 지정된 인덱스에 문서 저장
        3. 색인 실패 시 에러 로그를 기록하되 함수 실행은 계속 진행
        4. 비동기 처리를 통해 ES와의 네트워크 통신 최적화
        5. 에러가 발생해도 상위 호출자에게 성공으로 반환 (로그만 기록)

        # Arguments
        * `index_name` - 문서를 저장할 Elasticsearch 인덱스명
        * `alert_index` - 색인할 AlertIndex 구조체 참조

        # Returns
        * `()` - 항상 성공 반환 (에러는 로그로만 처리)
        * `anyhow::Error` - 실제로는 반환되지 않음 (내부에서 처리)
    "#]
    async fn post_log_index(
        &self,
        index_name: &str,
        alert_index: &AlertIndex,
    ) -> anyhow::Result<()> {
        self.es_conn
            .post_query_struct(alert_index, index_name)
            .await
            .unwrap_or_else(|e| {
                error!("[QueryServiceImpl->post_log_index] {:?}", e);
            });

        Ok(())
    }

    #[doc = ""]
    async fn post_alarm_history_index(
        &self,
        index_name: &str,
        alarm_history_index: AlarmLogHistoryIndex,
    ) -> anyhow::Result<()> {
        self.es_conn
            .post_query_struct(&alarm_history_index, index_name)
            .await
            .unwrap_or_else(|e| {
                error!(
                    "[QueryServiceImpl->post_alarm_history_index] {:?}",
                    e
                );
            });

        Ok(())
    }

    #[doc = r#"
        주어진 인덱스 설정(`IndexConfig`)과 기준 시각(`cur_timestamp_utc`)을 바탕으로
        이전 agg_term_sec 동안의 문서 수(`cnt`) 변동을 계산한다.

        1. `calc_time_window`를 통해 (기준시각 - agg_term_sec) ~ 기준시각 범위를 산출
        2. 해당 구간에서 `cnt` 필드의 최대/최소 값을 Elasticsearch 집계(`max`, `min`)로 조회
        3. 최소값을 기준으로 변화율(%)을 계산
        4. 변화율이 허용치(`allowable_fluctuation_range`) 이상이면,
        - 구간 내 데이터를 추가 조회하여 `AlertIndexFormat`으로 변환
        - 이를 포함한 `LogIndexResult`를 반환
        5. 변화율이 허용치 미만이면 `alert_info`가 None인 `LogIndexResult` 반환

        # Arguments
        * `mon_index_name` - 모니터링 정보를 가지고 있는 인덱스 이름
        * `index_config` - 모니터링 대상 인덱스 설정 (허용변동범위, 집계주기 포함)
        * `cur_timestamp_utc` - 기준 시각 (UTC, "%Y-%m-%dT%H:%M:%SZ" 포맷)

        # Returns
        * `LogIndexResult` - 인덱스명, 정상여부, (조건 충족 시) AlertIndexFormat 포함
        * `anyhow::Error` - ES 조회 실패 또는 파싱 실패 시
    "#]
    async fn get_alert_infos_from_log_index(
        &self,
        mon_index_name: &str,
        index_config: &IndexConfig,
        cur_timestamp_utc: DateTime<Utc>,
    ) -> anyhow::Result<LogIndexResult> {
        let index_name: &str = index_config.index_name();
        let allowable: f64 = *index_config.allowable_fluctuation_range();
        let agg_term: i64 = *index_config.agg_term_sec();

        let prev_timestamp_utc: DateTime<Utc> = calc_time_window(cur_timestamp_utc, agg_term);

        let (min_val, max_val) = self
            .fetch_min_max_values(
                mon_index_name,
                index_name,
                prev_timestamp_utc,
                cur_timestamp_utc,
            )
            .await?;

        let fluctuation_val: f64 = Self::calculate_fluctuation(min_val, max_val);
        let one_decimal: f64 = (fluctuation_val * 100.0).round() / 100.0; /* 소수점 첫째 자리까지만 표현 */

        let mut result: LogIndexResult =
            LogIndexResult::new(index_name.to_string(), false, None, one_decimal, 0);

        if fluctuation_val >= allowable {
            let sorts: Vec<SortSpec<'_>> = vec![SortSpec {
                field: "timestamp",
                order: SortOrder::Desc,
            }];

            let alert_index_formats: Vec<AlertIndexFormat> = self
                .fetch_index_cnt_infos(
                    mon_index_name,
                    index_name,
                    prev_timestamp_utc,
                    cur_timestamp_utc,
                    100,
                    Some(sorts),
                )
                .await?;

            let cur_index_cnt: usize = alert_index_formats
                .first()
                .map(|format| *format.alert_index().cnt())
                .unwrap_or_else(|| {
                    error!("[QueryServiceImpl->get_alert_infos_from_log_index] No alert index formats found");
                    0
                });

            let alert_indexes: Vec<AlertIndex> = alert_index_formats
                .into_iter()
                .map(|format| format.alert_index)
                .collect();

            result.set_alert_yn(true);
            result.set_cur_cnt(cur_index_cnt);
            result.set_alert_index_format(Some(alert_indexes));
        }

        Ok(result)
    }

    #[doc = ""]
    async fn get_report_infos_from_log_index(
        &self,
        mon_index_name: &str,
        index_name: &str,
        start_timestamp: DateTime<Utc>,
        end_timestamp: DateTime<Utc>,
    ) -> anyhow::Result<Vec<AlertIndex>> {
        let sorts: Vec<SortSpec<'_>> = vec![
            SortSpec {
                field: "timestamp",
                order: SortOrder::Asc,
            },
            /* 필요 시 보조 정렬도 추가 가능 */
            // SortSpec { field: "_score", order: SortOrder::Desc },
        ];

        let alert_index_formats: Vec<AlertIndexFormat> = self
            .fetch_index_cnt_infos(
                mon_index_name,
                index_name,
                start_timestamp,
                end_timestamp,
                10000,
                Some(sorts),
            )
            .await?;

        /***** !! 중요 - 한국시간으로 변환해서 저장해준다.!! ******/
        let report_indexes: Vec<AlertIndex> = alert_index_formats
            .into_iter()
            .map(|x| {
                let mut alert_index: AlertIndex = x.alert_index;
                let utc_to_local: String = calc_struct_to_strkor(alert_index.timestamp())
                    .unwrap_or(alert_index.timestamp().to_string());
                alert_index.set_timestamp(utc_to_local);

                alert_index
            })
            .collect();

        Ok(report_indexes)
    }
}
