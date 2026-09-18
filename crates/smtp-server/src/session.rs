// smtp-server/src/session.rs
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use mail_parser::Message;
use nanoid::nanoid;

use apiServer::AppState;
use storage::{store_email_in_redis, inbox_exists};
use cores::types::{Email, MAXIMUM_SIZE, SmtpState, SUPPORTED_EXTENSIONS};

pub async fn handle_connection(
    mut socket: TcpStream,
    app_state: AppState,
) -> anyhow::Result<()> {

    socket.write_all(b"220 TempMail Ready\r\n").await?;

    let mut smtp_state = SmtpState::Connected;

    let mut reader = BufReader::new(socket);

    let mut line = String::new();

    loop {

        line.clear();

        let bytes = reader.read_line(&mut line).await?;

        if bytes == 0 {
            return Ok(());
        }

        let command = line.trim();

        match &mut smtp_state {

            SmtpState::Connected => {

                if command.starts_with("EHLO") || command.starts_with("HELO") {

                    let mut response = String::from("250-temp-mail\r\n");

                    for ext in SUPPORTED_EXTENSIONS {
                        response.push_str(&format!("250-{ext}\r\n"));
                    }

                    response.push_str("250 OK\r\n");

                    reader.get_mut().write_all(response.as_bytes()).await?;

                    smtp_state = SmtpState::Greeted;

                } else {

                    reader.get_mut().write_all(b"500 Expected EHLO/HELO\r\n").await?;
                }
            }

            SmtpState::Greeted => {

                if command.starts_with("MAIL FROM:") {

                    let sender = command[10..].trim().to_string();

                    reader.get_mut().write_all(b"250 OK\r\n").await?;

                    smtp_state = SmtpState::MailFrom(sender);

                } else {

                    reader.get_mut().write_all(b"500 Expected MAIL FROM\r\n").await?;
                }
            }

            SmtpState::MailFrom(sender) => {

                if command.starts_with("RCPT TO:") {

                    let recipient = command[8..].trim();

                    if !inbox_exists(&app_state, recipient).await? {

                        reader.get_mut().write_all(b"550 Inbox not found\r\n").await?;
                        continue;
                    }

                    reader.get_mut().write_all(b"250 OK\r\n").await?;

                    smtp_state = SmtpState::RcptTo {
                        sender: sender.clone(),
                        recipients: vec![recipient.to_string()],
                    };

                } else {

                    reader.get_mut().write_all(b"500 Expected RCPT TO\r\n").await?;
                }
            }

            SmtpState::RcptTo { sender, recipients } => {

                if command == "DATA" {

                    reader.get_mut().write_all(b"354 End with <CRLF>.<CRLF>\r\n").await?;

                    let mut data = Vec::new();
                    let mut data_line = String::new();

                    loop {

                        data_line.clear();

                        let bytes = reader.read_line(&mut data_line).await?;

                        if bytes == 0 {
                            break;
                        }

                        if data_line == ".\r\n" {
                            break;
                        }

                        if data.len() + data_line.len() > MAXIMUM_SIZE {

                            reader.get_mut().write_all(b"552 Message too large\r\n").await?;
                            return Ok(());
                        }

                        data.extend_from_slice(data_line.as_bytes());
                    }

                    let parsed = Message::parse(&data)
                        .ok_or_else(|| anyhow::anyhow!("Invalid MIME message"))?;

                    let email = Email {

                        id: nanoid!(),

                        sender: sender.clone(),

                        recipients: recipients.clone(),

                        subject: parsed.subject().unwrap_or("").into(),

                        body_text: parsed.body_text(0).unwrap_or("").into(),

                        body_html: parsed.body_html(0).unwrap_or("").into(),

                        received_at: chrono::Utc::now().timestamp(),
                    };

                    store_email_in_redis(&app_state, &email).await
                        .map_err(|_| anyhow::anyhow!("Redis store failed"))?;

                    reader.get_mut().write_all(b"250 OK\r\n").await?;

                    smtp_state = SmtpState::Done;

                } else {

                    reader.get_mut().write_all(b"500 Expected DATA\r\n").await?;
                }
            }

            SmtpState::Done => {

                if command == "QUIT" {

                    reader.get_mut().write_all(b"221 Bye\r\n").await?;

                    return Ok(());

                } else {

                    reader.get_mut().write_all(b"500 Expected QUIT\r\n").await?;
                }
            }
        }
    }
}