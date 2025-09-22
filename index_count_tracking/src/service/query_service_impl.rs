use crate::common::*;

use crate::traits::{repository_traits::es_repository::*, service_traits::query_service::*};

use crate::repository::es_repository_impl::*;

// use crate::utils_modules::time_utils::*;
use crate::utils_modules::{
    traits::*,
    time_utils::*
};

use crate::model::{
    index::{alert_index::*, index_config::*},
    alarm::alarm_index_infos::*
};

// use crate::model::{
//     error_alarm_info::*, error_alarm_info_format::*, vector_index_log::*,
//     vector_index_log_format::*,
// };

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
                anyhow!("[QueryServicePub->get_query_result_vec] Missing 'hits.hits' field")
            })?;

        let arr: &Vec<Value> = hits.as_array().ok_or_else(|| {
            anyhow!("[QueryServicePub->get_query_result_vec] 'hits.hits' is not an array")
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
                        anyhow!("[QueryServicePub->get_query_result_vec] Missing or invalid '_id'")
                    })?
                    .to_string();

                /* 2) source 역직렬화 */
                let src_val: &Value = hit.get("_source").ok_or_else(|| {
                    anyhow!("[QueryServicePub->get_query_result_vec] Missing '_source'")
                })?;

                let source: S = serde_json::from_value(src_val.clone()).map_err(|e| {
                    anyhow!(
                        "[QueryServicePub->get_query_result_vec] Failed to deserialize source: {}",
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
                anyhow!("[QueryServicePub->get_query_result] Missing 'hits.hits' field")
            })?;

        let arr: &Vec<Value> = hits.as_array().ok_or_else(|| {
            anyhow!("[QueryServicePub->get_query_result] 'hits.hits' is not an array")
        })?;

        let first_hit: &Value = arr
            .first()
            .ok_or_else(|| anyhow!("[QueryServicePub->get_query_result] hits array is empty"))?;

        let id: String = first_hit
            .get("_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("[QueryServicePub->get_query_result] Missing or invalid '_id'"))?
            .to_string();

        let src_val: &Value = first_hit
            .get("_source")
            .ok_or_else(|| anyhow!("[QueryServicePub->get_query_result] Missing '_source'"))?;

        let source: S = serde_json::from_value(src_val.clone()).map_err(|e| {
            anyhow!(
                "[QueryServicePub->get_query_result] Failed to deserialize source: {}",
                e
            )
        })?;

        Ok(T::from_search_hit(id, source))
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
    async fn post_log_index(&self, index_name: &str, alert_index: &AlertIndex) -> anyhow::Result<()> {
        
        self.es_conn.post_query_struct(alert_index, index_name).await
            .unwrap_or_else(|e| {
                error!("[QueryServiceImpl->post_log_index] {:?}", e);
            });
        
        Ok(())
    }
    
    #[doc = ""]
    async fn get_max_cnt_from_log_index(&self, index_config: &IndexConfig, cur_timestamp_utc: &str) -> anyhow::Result<()> {

        let index_name: &str = index_config.index_name();
        let allowable_fluctuation_range: usize = *index_config.allowable_fluctuation_range();
        let agg_term: i64 = *index_config.agg_term_sec();
        
        let prev_timestamp_utc: String = calc_time_window(cur_timestamp_utc, agg_term)?;

        let max_query: Value = json!({
            "size": 0,
            "track_total_hits": false,
            "query": {
                "range": {
                    "timestamp": {
                        "gte": prev_timestamp_utc,
                        "lte": cur_timestamp_utc
                    }
                },
                "aggs": {
                    "max_value_in_range": {
                        "max": {
                            "field": "cnt"
                        }
                    }
                }
            }
        });

        let max_resp: Value = self.es_conn.get_search_query(&max_query, index_name).await?;
        let maybe_max_val: f64 = max_resp["aggregations"]["max_value_in_range"]["value"].as_f64().unwrap_or(0.0);


        let min_query: Value = json!({
            "size": 0,
            "track_total_hits": false,
            "query": {
                "range": {
                    "timestamp": {
                        "gte": prev_timestamp_utc,
                        "lte": cur_timestamp_utc
                    }
                },
                "aggs": {
                    "min_value_in_range": {
                        "min": {
                            "field": "cnt"
                        }
                    }
                }
            }
        });


        let min_resp: Value = self.es_conn.get_search_query(&max_query, index_name).await?;
        let maybe_min_val: f64 = max_resp["aggregations"]["max_value_in_range"]["value"].as_f64().unwrap_or(0.0);
        
        let fluctuation_val: f64 = if maybe_min_val > 0.0 {
            ((maybe_max_val - maybe_min_val) / maybe_min_val) * 100.0
        } else {
            0.0  
        };
        
        Ok(())
    }

    // #[doc = "색인 동작 로그를 가져오는 함수"]
    // /// # Arguments
    // /// * `query_index` - 쿼리의 대상이 되는 Elasticsearch 인덱스 이름
    // /// * `index_name`  - 색인될 인덱스의 이름
    // /// * `index_type`  - 정적색인인지 동적색인인지 구분하는 타입
    // /// * `start_dt`    - 색인 시작 시각
    // /// * `end_dt`      - 색인 종료 시각
    // ///
    // /// # Returns
    // /// * Result<Vec<VectorIndexLog>, anyhow::Error>
    // async fn get_indexing_movement_log(
    //     &self,
    //     query_index: &str,
    //     index_name: &str,
    //     index_type: &str,
    //     start_dt: NaiveDateTime,
    //     end_dt: NaiveDateTime,
    // ) -> Result<VectorIndexLogFormat, anyhow::Error> {
    //     let start_dt_str: String = get_str_from_naive_datetime(start_dt, "%Y-%m-%dT%H:%M:%SZ")?;
    //     let end_dt_str: String = get_str_from_naive_datetime(end_dt, "%Y-%m-%dT%H:%M:%SZ")?;

    //     let query: Value = json!({
    //         "size": 1,                       /* 최신 한 건만 */
    //         "track_total_hits": false,       /* 총건수 집계 불필요 - 성능상 좋음 */
    //         "query": {
    //             "bool": {
    //                 "filter": [
    //                     { "term":  { "index_name.keyword": index_name } },
    //                     { "term":  { "state.keyword":      index_type } },
    //                     { "range": { "timestamp": {
    //                         "gte": start_dt_str,
    //                         "lte": end_dt_str
    //                     }}},
    //                     { "match_phrase": { "message": "index worked" } }
    //                 ]
    //             }
    //         },
    //         "sort": [
    //             { "timestamp": { "order": "desc" } } /* 최신순 */
    //         ]
    //     });

    //     let es_client: ElasticConnGuard = get_elastic_guard_conn().await?;
    //     let response_body: Value = es_client.get_search_query(&query, query_index).await?;

    //     let result: VectorIndexLogFormat =
    //         self.get_query_result::<VectorIndexLogFormat, VectorIndexLog>(&response_body)?;

    //     Ok(result)
    // }

    // #[doc = "색인 실패 정보를 모니터링 Elasitcsearch 인덱스에 색인해주는 함수"]
    // /// # Arguments
    // /// * `index_name`  - 에러메시지 정보가 들어있는 인덱스 이름
    // ///
    // /// # Returns
    // /// * Result<(), anyhow::Error>
    // async fn post_indexing_error_info(
    //     &self,
    //     index_name: &str,
    //     error_alaram_info: ErrorAlarmInfo,
    // ) -> Result<(), anyhow::Error> {
    //     let es_client: ElasticConnGuard = get_elastic_guard_conn().await?;

    //     es_client
    //         .post_query_struct(&error_alaram_info, index_name)
    //         .await?;

    //     Ok(())
    // }

    // #[doc = "색인 에러 정보들을 반환해주는 함수"]
    // /// # Arguments
    // /// * `index_name`  - 에러메시지 정보가 들어있는 인덱스 이름
    // ///
    // /// # Returns
    // /// * Result<Vec<ErrorAlarmInfo>, anyhow::Error>
    // async fn get_error_alarm_infos(
    //     &self,
    //     index_name: &str,
    // ) -> Result<Vec<ErrorAlarmInfoFormat>, anyhow::Error> {
    //     let es_client: ElasticConnGuard = get_elastic_guard_conn().await?;

    //     let query: Value = json!({
    //         "query": {
    //             "match_all": {}
    //         },
    //         "size": 1000
    //     });

    //     let response_body: Value = es_client.get_search_query(&query, index_name).await?;
    //     let err_alram_infos: Vec<ErrorAlarmInfoFormat> =
    //         self.get_query_result_vec::<ErrorAlarmInfoFormat, ErrorAlarmInfo>(&response_body)?;

    //     Ok(err_alram_infos)
    // }

    // #[doc = "특정 인덱스의 특정 문서를 삭제해주는 함수"]
    // /// # Arguments
    // /// * `index_name` - 삭제 대상이 되는 인덱스 이름
    // /// * `doc_id` - 삭제할 문서의 id
    // ///
    // /// # Returns
    // /// * Result<Vec<ErrorAlarmInfo>, anyhow::Error>
    // async fn delete_index_by_doc(
    //     &self,
    //     index_name: &str,
    //     doc_id: &str,
    // ) -> Result<(), anyhow::Error> {
    //     let es_client: ElasticConnGuard = get_elastic_guard_conn().await?;
    //     es_client.delete_query(doc_id, index_name).await?;
    //     Ok(())
    // }
}
