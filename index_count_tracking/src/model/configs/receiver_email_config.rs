use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct ReceiverEmail {
    pub email_id: String,
}

#[derive(Serialize, Deserialize, Debug, Getters)]
pub struct ReceiverEmailConfig {
    pub emails: Vec<ReceiverEmail>,
}