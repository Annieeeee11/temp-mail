// crates/cores/src/lib.rs

// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::TcpStream;

// use mail_parser::Message;
// use anyhow::{Result};
pub mod types;
pub mod error;

// use apiServer::AppState;
// use storage::lib::{store_email_in_redis, inbox_exists};
// use types::{Email, SmtpState, MAXIMUM_SIZE, SUPPORTED_EXTENSIONS};

// pub async fn handle_connection(
//     mut socket: TcpStream,
//     state: AppState,
// ) -> anyhow::Result<()> {

//     socket.write_all(b"220 TempMail Ready\r\n").await?;

//     let mut state = SmtpState::Connected;

//     let mut buffer = vec![0; MAXIMUM_SIZE];

//     loop {

//         let n = socket.read(&mut buffer).await?;

//         if n == 0 {
//             return Ok(());
//         }

//         let line = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

//         match &mut state {

//             SmtpState::Connected => {

//                 if line.starts_with("EHLO") {

//                     let mut response = String::from("250-temp-mail\r\n");

//                     for ext in SUPPORTED_EXTENSIONS {
//                         response.push_str(&format!("250-{ext}\r\n"));
//                     }

//                     response.push_str("250 OK\r\n");

//                     socket.write_all(response.as_bytes()).await?;

//                     state = SmtpState::Greeted;
//                 }
//             }

//             SmtpState::Greeted => {

//                 if line.starts_with("MAIL FROM:") {

//                     let sender = line[10..].trim().to_string();

//                     socket.write_all(b"250 OK\r\n").await?;

//                     state = SmtpState::MailFrom(sender);
//                 }
//             }

//             SmtpState::MailFrom(sender) => {

//                 if line.starts_with("RCPT TO:") {

//                     let recipient = line[8..].trim();

//                     if !inbox_exists(&state, recipient).await? {

//                         socket.write_all(b"550 Inbox not found\r\n").await?;
//                         continue;
//                     }

//                     socket.write_all(b"250 OK\r\n").await?;

//                     state = SmtpState::RcptTo {
//                         sender: sender.clone(),
//                         recipients: vec![recipient.to_string()],
//                     };
//                 }
//             }

//             SmtpState::RcptTo { sender, recipients } => {

//                 if line == "DATA" {

//                     socket.write_all(b"354 End with <CRLF>.<CRLF>\r\n").await?;

//                     let mut data = Vec::new();

//                     loop {

//                         let n = socket.read(&mut buffer).await?;

//                         if n == 0 {
//                             break;
//                         }

//                         data.extend_from_slice(&buffer[..n]);

//                         if data.ends_with(b"\r\n.\r\n") {
//                             break;
//                         }

//                         if data.len() > MAXIMUM_SIZE {
//                             socket.write_all(b"552 Message too large\r\n").await?;
//                             return Ok(());
//                         }
//                     }

//                     let parsed = Message::parse(&data).ok_or_else(|| anyhow::anyhow!("Invalid MIME message"))?;

//                     let email = Email {
//                         sender: sender.clone(),
//                         recipients: recipients.clone(),
//                         subject: parsed.subject().unwrap_or("").into(),
//                         body_text: parsed.body_text(0).unwrap_or("").into(),
//                         body_html: parsed.body_html(0).unwrap_or("").into(),
//                     };

//                     store_email_in_redis(&state, &email).await?;

//                     socket.write_all(b"250 OK\r\n").await?;

//                     state = SmtpState::Done;
//                 }
//             }

//             SmtpState::Done => {

//                 if line == "QUIT" {

//                     socket.write_all(b"221 Bye\r\n").await?;

//                     return Ok(());
//                 }
//             }
//         }
//     }
// }