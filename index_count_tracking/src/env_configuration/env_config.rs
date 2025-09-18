use crate::common::*;

#[doc = "환경변수를 읽고, 없을 경우 error 로그 후 panic"]
fn get_env_or_panic(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => {
            let msg = format!("[ENV file read Error] '{}' must be set", key);
            error!("{}", msg);
            panic!("{}", msg);
        }
    }
}

#[doc = "Function to globally initialize the 'INDEX_LIST_PATH' variable"]
pub static INDEX_LIST_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("INDEX_LIST_PATH"));

#[doc = "Function to globally initialize the 'EMAIL_RECEIVER_PATH' variable"]
pub static EMAIL_RECEIVER_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("EMAIL_RECEIVER_PATH"));

#[doc = "Function to globally initialize the 'SERVER_CONFIG_PATH' variable"]
pub static SERVER_CONFIG_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("SERVER_CONFIG_PATH"));

#[doc = "Function to globally initialize the 'HTML_TEMPLATE_PATH' variable"]
pub static HTML_TEMPLATE_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("HTML_TEMPLATE_PATH"));