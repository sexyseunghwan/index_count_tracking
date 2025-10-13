/*
Author      : Seunghwan Shin
Create date : 2025-09-24
Description : 색인되고 있는 인덱스 개수의 현황을 파악하고 변화율이 높으면 알람을 보내주는 프로그램

History     : 2025-09-24 Seunghwan Shin       # [v.1.0.0] first create
              2025-10-00 Seunghwan Shin       # [v.2.0.0] 정기적으로 특정 시간에 지난 24시간 공고수 추이 리포트를 메일로 보내주는 기능 추가
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
use service::{
    chart_service_impl::*, report_service_impl::*, notification_service_impl::*,
    query_service_impl::*, tracking_monitor_service_impl::*,
};

mod controller;
use controller::main_controller::*;

use crate::controller::main_controller;

mod dto;

mod enums;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    /* 전역로거 설정 및 초기 설정 */
    dotenv().ok();
    set_global_logger();

    info!("Tracking program start!");

    /* Elasticsearch connection */
    /* 모니터링 대상 Elasticsearch cluster conneciton */
    let target_es_conn: Arc<EsRepositoryImpl> = Arc::new(
        EsRepositoryImpl::new(get_elastic_config_info()).map_err(|e| {
            let err_msg = "[main] An issue occurred while initializing target_es_conn.";
            error!("{} {:?}", err_msg, e);
            anyhow!("{} {:?}", err_msg, e)
        })?,
    );

    /* 모니터링용 Elasticsearch cluster conneciton */
    let mon_es_conn: Arc<EsRepositoryImpl> = Arc::new(
        EsRepositoryImpl::new(get_mon_elastic_config_info()).map_err(|e| {
            let err_msg: &'static str = "[main] An issue occurred while initializing mon_es_conn.";
            error!("{} {:?}", err_msg, e);
            anyhow!("{} {:?}", err_msg, e)
        })?,
    );

    /* 의존 주입 */
    let notification_service: Arc<NotificationServiceImpl> =
        Arc::new(NotificationServiceImpl::new()?);
        
    let tracking_monitor_service: TrackingServiceImpl<QueryServiceImpl, NotificationServiceImpl> =
        TrackingServiceImpl::new(
            QueryServiceImpl::new(Arc::clone(&target_es_conn)),
            QueryServiceImpl::new(Arc::clone(&mon_es_conn)),
            Arc::clone(&notification_service),
        );

    let chart_service: ChartServiceImpl = ChartServiceImpl::new();
    let daily_report_service: ReportServiceImpl<
        QueryServiceImpl,
        ChartServiceImpl,
        NotificationServiceImpl,
    > = ReportServiceImpl::new(
        QueryServiceImpl::new(Arc::clone(&mon_es_conn)),
        chart_service,
        Arc::clone(&notification_service),
    );

    let main_controller = MainController::new(
        Arc::new(tracking_monitor_service),
        Arc::new(daily_report_service)
    );

    if let Err(e) = main_controller.main_task().await {
        error!("Main task failed: {:?}", e);
        return Err(e);
    }

    Ok(())
}
