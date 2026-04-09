#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SMTP network error: {0}")]
    NetworkError(#[from] std::io::Error),

    #[error("mail from missing")]
    MailFromMissing,

    #[error("rcpt to missing")]
    RcptToMissing,

    #[error("data missing")]
    DataMissing,
}