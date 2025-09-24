use crate::common::*;

use crate::traits::{repository_traits::es_repository::*, service_traits::query_service::*};

use crate::repository::es_repository_impl::*;

use crate::utils_modules::{time_utils::*, traits::*};

use crate::model::index::{alert_index::*, alert_index_format::*, index_config::*};

use crate::dto::log_index_result::*;


#[derive(Debug, new)]
pub struct QueryServiceImpl {
    es_conn: EsRepositoryImpl,
}

impl QueryServiceImpl {
    #[doc = "Functions that return queried results as vectors"]
    /// # Arguments
    /// * `response_body` - Querying Results
    ///
    /// # Returns
    /// * Result<Vec<T>, anyhow::Error>
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

    #[doc = "Functions that return queried results"]
    /// # Arguments
    /// * `response_body` - Querying Results
    ///
    /// # Returns
    /// * Result<T, anyhow::Error>
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

    #[doc = "주어진 시간 범위(gte~lte)에 대해 단일 집계 값을 f64로 받아오는 헬퍼"]
    async fn fetch_agg_value_f64(
        &self,
        mon_index_name: &str,
        index_name: &str,
        gte: &str,
        lte: &str,
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
                                "timestamp": { "gte": gte, "lte": lte }
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

    async fn fetch_min_max_values(
        &self,
        mon_index_name: &str,
        index_name: &str,
        gte: &str,
        lte: &str,
    ) -> anyhow::Result<(f64, f64)> {
        let max_val = self
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

        let min_val = self
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

    fn calculate_fluctuation(min_val: f64, max_val: f64) -> f64 {
        if min_val > 0.0 {
            ((max_val - min_val) / min_val) * 100.0
        } else {
            0.0
        }
    }

    async fn fetch_alert_data(
        &self,
        mon_index_name: &str,
        index_name: &str,
        prev_timestamp_utc: &str,
        cur_timestamp_utc: &str,
    ) -> anyhow::Result<Vec<AlertIndexFormat>> {
        let search_query: Value = json!({
            "query": {
                "bool": {
                    "filter": [
                        {
                            "range": {
                                "timestamp": { "gte": prev_timestamp_utc, "lte": cur_timestamp_utc }
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
            "sort": [{ "timestamp": { "order": "desc" } }]
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
    #[doc = "특정 인덱스의 총 문서 개수 반환해주는 함수"]
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

    #[doc = "로그 인덱스를 색인해주는 함수"]
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
        cur_timestamp_utc: &str,
    ) -> anyhow::Result<LogIndexResult> {
        let index_name: &str = index_config.index_name();
        let allowable: f64 = *index_config.allowable_fluctuation_range();
        let agg_term: i64 = *index_config.agg_term_sec();

        let prev_timestamp_utc: String = calc_time_window(cur_timestamp_utc, agg_term)?;

        let (min_val, max_val) = self
            .fetch_min_max_values(mon_index_name, index_name, &prev_timestamp_utc, cur_timestamp_utc)
            .await?;

        let fluctuation_val: f64 = Self::calculate_fluctuation(min_val, max_val);

        let mut result: LogIndexResult = LogIndexResult::new(index_name.to_string(), false, None, fluctuation_val, 0);

        if fluctuation_val >= allowable {
            let alert_index_formats: Vec<AlertIndexFormat> = self
                .fetch_alert_data(mon_index_name, index_name, &prev_timestamp_utc, cur_timestamp_utc)
                .await?;

            let cur_index_cnt: usize = alert_index_formats
                .first()
                .map(|format| *format.alert_index().cnt())
                .unwrap_or_else(|| {
                    error!("[QueryServiceImpl->get_alert_infos_from_log_index] No alert index formats found");
                    0
                });

            result.set_cur_cnt(cur_index_cnt);

            let alert_indexes: Vec<AlertIndex> = alert_index_formats
                .into_iter()
                .map(|format| format.alert_index)
                .collect();

            result.set_alert_index_format(Some(alert_indexes));
        }

        Ok(result)
    }
}
