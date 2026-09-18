// crates/cores/src/types.rs
use const_format::concatcp;
use serde::{Serialize, Deserialize};

pub const MAXIMUM_SIZE: usize = 15_728_640; // 15 mb

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EmailBody {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Email {
    pub id: String,

    pub sender: String,
    pub recipients: Vec<String>,

    pub subject: String,
    pub body_text: String,
    pub body_html: String,

    pub received_at: i64,
}

#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub esmtp: bool,
    pub greeting_done: bool,
    pub session_closed: bool,
    pub mail_from: Option<String>,
    pub rcpt_to: Vec<String>,
    pub waiting_for_data: bool,
    pub data: EmailBody,
}

pub struct Inbox {
    pub address: String,
    pub messages: Vec<Email>,
}

// pub struct MessageHeader {
//     pub from: String,
//     pub to: Vec<String>,
//     pub subject: Option<String>,
//     pub date: Option<String>,
// }

#[derive(Debug, Clone)]
pub enum Event {
    EmailReceived(Email),
}
pub type EventHandler = fn(Event);

pub const SUPPORTED_EXTENSIONS: &[&str; 1] = &[
    concatcp!("SIZE ", MAXIMUM_SIZE),
];
#[derive(Debug)]
pub enum SmtpState {
    Connected,
    Greeted,
    MailFrom(String),
    RcptTo {
        sender: String,
        recipients: Vec<String>,
    },
    Data {
        sender: String,
        recipients: Vec<String>,
    },
    Done,
}