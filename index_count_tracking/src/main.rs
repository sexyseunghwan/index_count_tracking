/*
Author      : Seunghwan Shin
Create date : 2025-09-24
Description : 색인되고 있는 인덱스 개수의 현황을 파악하고 변화율이 높으면 알람을 보내주는 프로그램

History     : 2025-09-24 Seunghwan Shin       # [v.1.0.0] first create
*/

mod common;
mod external_deps;
mod prelude;
use common::*;

mod repository;
use repository::es_repository_impl::*;

mod env_configuration;

mod traits;

mod model;
use model::configs::total_config::*;

mod utils_modules;
use utils_modules::logger_utils::*;

mod service;
use service::{notification_service_impl::*, query_service_impl::*};

mod controller;
use controller::main_controller::*;

mod dto;

#[tokio::main]
async fn main() {
    /* 전역로거 설정 및 초기 설정 */
    dotenv().ok();
    set_global_logger();

    info!("Tracking program start!");

    /* Elasticsearch connection */
    /* 모니터링 대상 Elasticsearch cluster conneciton */
    let target_es_conn: EsRepositoryImpl = EsRepositoryImpl::new(get_elastic_config_info())
        .unwrap_or_else(|e| {
            let err_msg: &str = "[main] An issue occurred while initializing target_es_conn.";
            error!("{} {:?}", err_msg, e);
            panic!("{} {:?}", err_msg, e)
        });
    
    /* 모니터링용 Elasticsearch cluster conneciton */
    let mon_es_conn: EsRepositoryImpl = EsRepositoryImpl::new(get_mon_elastic_config_info())
        .unwrap_or_else(|e| {
            let err_msg: &str = "[main] An issue occurred while initializing mon_es_conn.";
            error!("{} {:?}", err_msg, e);
            panic!("{} {:?}", err_msg, e)
        });

    /* 의존 주입 */
    let target_query_service: QueryServiceImpl = QueryServiceImpl::new(target_es_conn);
    let mon_query_service: QueryServiceImpl = QueryServiceImpl::new(mon_es_conn);
    let notification_service: NotificationServiceImpl = NotificationServiceImpl::new();

    let main_controller: MainController<
        NotificationServiceImpl,
        QueryServiceImpl,
        QueryServiceImpl,
    > = MainController::new(
        notification_service,
        target_query_service,
        mon_query_service,
    );

    main_controller.main_task().await.unwrap_or_else(|e| {
        error!("{:?}", e);
        panic!("{:?}", e)
    });
}
