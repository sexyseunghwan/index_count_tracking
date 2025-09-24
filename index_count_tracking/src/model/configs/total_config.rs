use crate::common::*;

use crate::model::configs::{
    elastic_server_config::*, rdb_config::*, system_config::*, telegram_config::*, smtp_config::*
};

use crate::utils_modules::io_utils::*;

use crate::env_configuration::env_config::*;

static TOTAL_CONFIG: once_lazy<TotalConfig> = once_lazy::new(|| initialize_server_config());

#[doc = "Function to initialize Server configuration information instances"]
pub fn initialize_server_config() -> TotalConfig {
    info!("initialize_server_config() START!");
    TotalConfig::new()
}

#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct TotalConfig {
    pub elasticsearch: ElasticServerConfig,
    pub mon_elasticsearch: ElasticServerConfig,
    pub sqlserver: RdbConfig,
    pub telegram: TelegramConfig,
    pub system: SystemConfig,
    pub smtp: SmtpConfig
}

#[doc = "Elasticsearch config 정보"]
pub fn get_elastic_config_info() -> &'static ElasticServerConfig {
    &TOTAL_CONFIG.elasticsearch
}

#[doc = "모니터링용 Elasticsearch config 정보"]
pub fn get_mon_elastic_config_info() -> &'static ElasticServerConfig {
    &TOTAL_CONFIG.mon_elasticsearch
}

#[doc = "알람용 Telegram 정보"]
pub fn get_telegram_config_info() -> &'static TelegramConfig {
    &TOTAL_CONFIG.telegram
}

#[doc = "Sql Server 설정 정보"]
pub fn get_sqlserver_config_info() -> &'static RdbConfig {
    &TOTAL_CONFIG.sqlserver
}

#[doc = "system 설정 정보"]
pub fn get_system_config_info() -> &'static SystemConfig {
    &TOTAL_CONFIG.system
}

#[doc = "system 설정 정보"]
pub fn get_smtp_config_info() -> &'static SmtpConfig {
    &TOTAL_CONFIG.smtp
}

impl TotalConfig {
    fn new() -> Self {
        read_toml_from_file::<TotalConfig>(&SERVER_CONFIG_PATH)
                .unwrap_or_else(|e| {
                    let err_msg: &str = "Failed to convert the data from SERVER_CONFIG_PATH into the TotalConfig structure.";
                    error!("[TotalConfig->new] {} {:?}", err_msg, e);
                    panic!("[TotalConfig->new] {} {:?}", err_msg, e)
                })
    }
}
