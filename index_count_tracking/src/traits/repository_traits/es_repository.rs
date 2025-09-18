use crate::common::*;

#[async_trait]
pub trait EsRepository {
    async fn get_search_query(
        &self,
        es_query: &Value,
        index_name: &str,
    ) -> Result<Value, anyhow::Error>;
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error>;
    async fn post_query_struct<T: Serialize + Sync>(
        &self,
        param_struct: &T,
        index_name: &str,
    ) -> Result<(), anyhow::Error>;
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error>;
}
