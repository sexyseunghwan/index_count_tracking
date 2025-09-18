use crate::common::*;

#[async_trait]
pub trait SqlServerRepository {
    #[doc = "SQL Server 아이메일러 관련 프로시저 호출"]
    async fn execute_imailer_procedure(
        &self,
        send_email: &str,
        email_subject: &str,
        email_content: &str,
    ) -> Result<(), anyhow::Error>;
}
