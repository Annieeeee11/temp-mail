use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("SMTP network error: {0}")]
    NetworkError(#[from] std::io::Error),

    #[error("mail from missing")]
    MailFromMissing,

    #[error("rcpt to missing")]
    RcptToMissing,

    #[error("data missing")]
    DataMissing,

    #[error("invalid command")]
    InvalidCommand,
    
    #[error("message too long")]
    MessageTooLarge,
}