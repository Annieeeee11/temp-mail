#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Email {
    pub sender: String,
    pub recipients: Vec<String>,
    pub data: String,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub esmtp: bool,
    pub greeting_done: bool,
    pub finished: bool,
    pub mail_from: Option<String>,
    pub rcpt_to: Vec<String>,
    pub waiting_for_data: bool,
    pub data: Vec<u8>,
}

pub struct Inbox {}

pub struct MessageHeader {}
