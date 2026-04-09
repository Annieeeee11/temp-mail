use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sender Mail missing")]
    MailFromMissing,
    #[error("RCPT missing")]
    RcptToMissing,
    #[error("Data missing")]
    DataMissing,
    #[error("Invalid command args")]
    InvalidCommand,
    #[error("Message too long")]
    MessageTooLarge,
    #[error("SMTP network error {0}")]
    NetworkError(#[from] std::io::Error),
    #[error("Message not found")]
    InboxNotFound,
    #[error("Try again later")]
    RedisError,
}

impl Error {
    pub fn smtp_code(&self) -> u16 {
        match self {
            Error::NetworkError(_) => 451,
            Error::MailFromMissing => 503,
            Error::RcptToMissing => 503,
            Error::DataMissing => 503,
            Error::InvalidCommand => 500,
            Error::MessageTooLarge => 552,
            Error::InboxNotFound => 550,
            Error::RedisError => 451,
        }
    }

    pub fn smtp_response(&self) -> String {
        format!("{} {}\r\n", self.smtp_code(), self)
    }
}

pub type SMTPResult<T> = Result<T, Error>;