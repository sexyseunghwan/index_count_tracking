use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::model::{configs::total_config::*, index::index_list_config::*};

use crate::env_configuration::env_config::*;

use crate::traits::service_traits::{report_service::*, tracking_monitor_service::*};

use crate::enums::report_type::*;

#[derive(Debug, new)]
pub struct MainController<T: TrackingMonitorService, R: ReportService> {
    tracking_monitor_service: Arc<T>,
    report_service: Arc<R>,
}

impl<T, R> MainController<T, R>
where
    T: TrackingMonitorService + Send + Sync + 'static,
    R: ReportService + Send + Sync + 'static,
{
    #[doc = r#"
        메인 루프를 실행하는 핵심 함수로, 30초 간격으로 인덱스 모니터링 작업을 반복 수행한다.

        1. 인덱스 설정 파일(`INDEX_LIST_PATH`)을 읽어와 모니터링 대상 인덱스 목록을 가져온다
        2. 30초마다 다음 작업들을 순차적으로 실행:
           - `save_index_cnt_infos`: 각 인덱스의 현재 문서 개수를 모니터링 인덱스에 저장
           - `verify_index_cnt`: 저장된 데이터를 바탕으로 각 인덱스의 문서 개수 변동을 검증
           - `alert_index_status`: 변동이 허용 범위를 초과한 인덱스에 대해 알람 발송
        3. 무한루프로 동작하며, 각 단계에서 오류 발생 시 해당 사이클을 중단하고 다음 사이클로 진행

        # Returns
        * `anyhow::Result<()>` - 정상 종료 시 Ok(()), 치명적 오류 시 Err
    "#]
    pub async fn main_task(&self) -> anyhow::Result<()> {
        let target_index_info_list: Arc<IndexListConfig> =
            Arc::new(read_toml_from_file::<IndexListConfig>(&INDEX_LIST_PATH)?);
        let mon_index_name: Arc<str> =
            Arc::from(get_system_config_info().monitor_index_name().to_string());
        let alarm_index_name: Arc<str> =
            Arc::from(get_alarm_log_index_info().index_name().to_string());

        /* 1. 모니터링 테스크 */
        let tracking_monitor_handle = Self::spawn_tracking_monitor_task(
            Arc::clone(&self.tracking_monitor_service),
            Arc::clone(&mon_index_name),
            Arc::clone(&target_index_info_list),
        );

        /* 2. 일일 리포트 테스크 */
        let daily_report_handle = Self::spawn_report_task(
            Arc::clone(&self.report_service),
            Arc::clone(&mon_index_name),
            Arc::clone(&alarm_index_name),
            Arc::clone(&target_index_info_list),
            ReportType::OneDay,
            "daily_report_task",
        );

        /* 3. 주간 리포트 테스크 */
        let weekly_report_handle = Self::spawn_report_task(
            Arc::clone(&self.report_service),
            Arc::clone(&mon_index_name),
            Arc::clone(&alarm_index_name),
            Arc::clone(&target_index_info_list),
            ReportType::OneWeek,
            "weekly_report_task",
        );

        /* 4. 월간 리포트 테스크 */
        let monthly_report_handle = Self::spawn_report_task(
            Arc::clone(&self.report_service),
            Arc::clone(&mon_index_name),
            Arc::clone(&alarm_index_name),
            Arc::clone(&target_index_info_list),
            ReportType::OneMonth,
            "monthly_report_task",
        );

        /* 모든 태스크를 병렬로 실행하고 종료를 대기 */
        let _ = tokio::join!(
            tracking_monitor_handle,
            daily_report_handle,
            weekly_report_handle,
            monthly_report_handle
        );

        Ok(())
    }

    #[doc = "모니터링 태스크를 별도의 tokio task로 spawn"]
    fn spawn_tracking_monitor_task(
        service: Arc<T>,
        mon_index_name: Arc<str>,
        target_index_info_list: Arc<IndexListConfig>,
    ) -> tokio::task::JoinHandle<()>
    where
        T: Send + Sync + 'static,
    {
        tokio::spawn(async move {
            match service
                .tracking_monitor_loop(&mon_index_name, &target_index_info_list)
                .await
            {
                Ok(_) => info!("[tracking_monitor_task] Completed successfully"),
                Err(e) => error!("[tracking_monitor_task] Failed with error: {:?}", e),
            }
        })
    }

    #[doc = "리포트 태스크를 별도의 tokio task로 spawn"]
    fn spawn_report_task(
        service: Arc<R>,
        mon_index_name: Arc<str>,
        alarm_index_name: Arc<str>,
        target_index_info_list: Arc<IndexListConfig>,
        report_type: ReportType,
        task_name: &str,
    ) -> tokio::task::JoinHandle<()>
    where
        R: Send + Sync + 'static,
    {
        let task_name: String = task_name.to_string();

        tokio::spawn(async move {
            match service
                .report_loop(
                    &mon_index_name,
                    &alarm_index_name,
                    &target_index_info_list,
                    report_type,
                )
                .await
            {
                Ok(_) => info!("[{}] Completed successfully", task_name),
                Err(e) => error!("[{}] Failed with error: {:?}", task_name, e),
            }
        })
    }
}
