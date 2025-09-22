use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::model::configs::{elastic_server_config::*, total_config::*};

use crate::traits::repository_traits::es_repository::*;

#[derive(Debug, Getters, Clone)]
pub struct EsRepositoryImpl {
    es_clients: Vec<EsClient>,
}

#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    es_conn: Elasticsearch,
}

impl EsRepositoryImpl {
    pub fn new(es_config: &ElasticServerConfig) -> Result<Self, anyhow::Error> {
        let mut es_clients: Vec<EsClient> = Vec::new();

        for url in &es_config.elastic_host {
            let parse_url: String = if let (Some(id), Some(pw)) = (
                es_config.elastic_id.as_deref(),
                es_config.elastic_pw.as_deref(),
            ) {
                format!("http://{}:{}@{}", id, encode(pw), url)
            } else {
                format!("http://{}", url)
            };

            let es_url: Url = Url::parse(&parse_url)?;
            let conn_pool: SingleNodeConnectionPool = SingleNodeConnectionPool::new(es_url);
            let transport: EsTransport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5, 0))
                .build()?;

            let elastic_conn: Elasticsearch = Elasticsearch::new(transport);
            let es_client: EsClient = EsClient::new(elastic_conn);

            es_clients.push(es_client);
        }

        Ok(EsRepositoryImpl { es_clients })
    }

    #[doc = "Common logic: common node failure handling and node selection"]
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(EsClient) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error: Option<anyhow::Error> = None;

        let mut rng: StdRng = StdRng::from_entropy();
        let mut shuffled_clients = self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng);

        for es_client in shuffled_clients {
            match operation(es_client).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        Err(anyhow::anyhow!(
            "All Elasticsearch nodes failed. Last error: {:?}",
            last_error
        ))
    }
}

#[async_trait]
impl EsRepository for EsRepositoryImpl {
    #[doc = "Function that EXECUTES elasticsearch queries - search"]
    async fn get_search_query(
        &self,
        es_query: &Value,
        index_name: &str,
    ) -> Result<Value, anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                let response = es_client
                    .es_conn
                    .search(SearchParts::Index(&[index_name]))
                    .body(es_query)
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let response_body: Value = response.json::<Value>().await?;
            Ok(response_body)
        } else {
            let error_body: String = response.text().await?;
            Err(anyhow!(
                "[Elasticsearch Error][node_search_query()] response status is failed: {:?}",
                error_body
            ))
        }
    }

    #[doc = "Function that EXECUTES elasticsearch queries - indexing struct"]
    async fn post_query_struct<T: Serialize + Sync>(
        &self,
        param_struct: &T,
        index_name: &str,
    ) -> Result<(), anyhow::Error> {
        let struct_json: Value = convert_json_from_struct(param_struct)?;
        self.post_query(&struct_json, index_name).await?;

        Ok(())
    }

    #[doc = "Function that EXECUTES elasticsearch queries - indexing"]
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                let response: Response = es_client
                    .es_conn
                    .index(IndexParts::Index(index_name))
                    .body(document)
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message = format!(
                "[Elasticsearch Error][node_post_query()] Failed to index document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Function that EXECUTES elasticsearch queries - delete"]
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error> {
        let response: Response = self
            .execute_on_any_node(|es_client| async move {
                let response: Response = es_client
                    .es_conn
                    .delete(DeleteParts::IndexId(index_name, doc_id))
                    .send()
                    .await?;

                info!(
                    "[{}] document of [{}] Index has been erased",
                    doc_id, index_name
                );

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message = format!(
                "[Elasticsearch Error][node_delete_query()] Failed to delete document: Status Code: {}, Document ID: {}",
                response.status_code(),
                doc_id
            );
            Err(anyhow!(error_message))
        }
    }
}
