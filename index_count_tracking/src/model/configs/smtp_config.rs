use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters)]
#[getset(get = "pub")]
pub struct SmtpConfig {
    pub smtp_name: String,
    pub credential_id: String,
    pub credential_pw: String,
    pub async_process_yn: bool
}
