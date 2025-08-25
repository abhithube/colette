use lettre::{
    AsyncSmtpTransport, AsyncTransport as _, Message, Tokio1Executor,
    address::AddressError,
    message::{
        Mailbox, Mailboxes,
        header::{self, ContentType},
    },
    transport::smtp::{self, authentication::Credentials},
};

pub trait SmtpClient: Sync {
    fn send(&self, email: SmtpEmail) -> impl Future<Output = Result<(), Error>> + Send;
}

#[derive(Debug, Clone)]
pub struct SmtpEmail {
    pub to_address: String,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct SmtpClientImpl {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpClientImpl {
    pub fn create(config: SmtpConfig) -> Result<Self, Error> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?
            .credentials(Credentials::new(config.username, config.password))
            .build();

        let from = config.from_address.parse::<Mailbox>()?;

        Ok(Self { transport, from })
    }
}

impl SmtpClient for SmtpClientImpl {
    async fn send(&self, email: SmtpEmail) -> Result<(), Error> {
        let message = Message::builder()
            .mailbox(header::From::from(Mailboxes::from(self.from.clone())))
            .to(email.to_address.parse()?)
            .subject(email.subject)
            .header(ContentType::TEXT_PLAIN)
            .body(email.body)?;

        self.transport.send(message).await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Transport(#[from] smtp::Error),

    #[error(transparent)]
    Address(#[from] AddressError),

    #[error(transparent)]
    Message(#[from] lettre::error::Error),
}
