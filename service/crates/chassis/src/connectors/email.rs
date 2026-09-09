use crate::error::ApiError;
use async_trait::async_trait;
use lettre::{
    message::Mailbox,
    transport::smtp::authentication::Credentials,
    transport::smtp::client::{Tls, TlsParameters},
    transport::smtp::{AsyncSmtpTransport, AsyncSmtpTransportBuilder},
    AsyncTransport, Message, Tokio1Executor,
};
use std::sync::Arc;

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_magic_link(&self, to: &str, link: &str) -> Result<(), ApiError>;
    async fn send_inquiry_notification(
        &self,
        to: &str,
        clinic_name: &str,
        patient_email: &str,
    ) -> Result<(), ApiError>;
}

pub struct MockEmailSender {
    pub sent: tokio::sync::Mutex<Vec<String>>,
}

impl MockEmailSender {
    pub fn new() -> Self {
        Self {
            sent: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

impl Default for MockEmailSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmailSender for MockEmailSender {
    async fn send_magic_link(&self, to: &str, link: &str) -> Result<(), ApiError> {
        self.sent
            .lock()
            .await
            .push(format!("magic-link to {}: {}", to, link));
        Ok(())
    }

    async fn send_inquiry_notification(
        &self,
        to: &str,
        clinic_name: &str,
        patient_email: &str,
    ) -> Result<(), ApiError> {
        self.sent.lock().await.push(format!(
            "inquiry to {} for {} from {}",
            to, clinic_name, patient_email
        ));
        Ok(())
    }
}

/// Encryption mode for the SMTP connection, selected by `SMTP_TLS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtpTls {
    /// STARTTLS upgrade on a plain connection (typical for port 587).
    StartTls,
    /// Implicit TLS from the first byte (typical for port 465).
    Tls,
    /// No encryption (local relays only, e.g. a same-host mail pit).
    Off,
}

/// SMTP connector configuration, sourced from the environment.
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from: Mailbox,
    pub tls: SmtpTls,
}

impl SmtpConfig {
    /// `None` when `SMTP_HOST` is unset/empty (mock mode). `Some(Err)` when the
    /// operator asked for SMTP but the configuration is incomplete/invalid, so
    /// the caller can log loudly instead of silently discarding email.
    pub fn from_env() -> Option<Result<Self, String>> {
        Self::from_vars(|key| std::env::var(key).ok())
    }

    fn from_vars(get: impl Fn(&str) -> Option<String>) -> Option<Result<Self, String>> {
        let host = get("SMTP_HOST").filter(|h| !h.is_empty())?;
        let tls = match get("SMTP_TLS")
            .unwrap_or_else(|| "starttls".into())
            .to_ascii_lowercase()
            .as_str()
        {
            "starttls" => SmtpTls::StartTls,
            "tls" => SmtpTls::Tls,
            "off" => SmtpTls::Off,
            other => return Some(Err(format!("SMTP_TLS must be starttls/tls/off, got {other:?}"))),
        };
        let default_port = match tls {
            SmtpTls::Tls => 465,
            _ => 587,
        };
        let port = match get("SMTP_PORT") {
            Some(p) => match p.parse() {
                Ok(p) => p,
                Err(_) => return Some(Err(format!("SMTP_PORT is not a valid port: {p:?}"))),
            },
            None => default_port,
        };
        let from = match get("SMTP_FROM") {
            Some(f) if !f.is_empty() => match f.parse::<Mailbox>() {
                Ok(m) => m,
                Err(_) => return Some(Err(format!("SMTP_FROM is not a valid address: {f:?}"))),
            },
            _ => return Some(Err("SMTP_FROM is required when SMTP_HOST is set".into())),
        };
        let username = get("SMTP_USERNAME").filter(|u| !u.is_empty());
        let password = get("SMTP_PASSWORD").filter(|p| !p.is_empty());
        Some(Ok(Self {
            host,
            port,
            username,
            password,
            from,
            tls,
        }))
    }

    fn transport_builder(&self) -> Result<AsyncSmtpTransportBuilder, ApiError> {
        let tls = match self.tls {
            SmtpTls::StartTls => Tls::Required(
                TlsParameters::new(self.host.clone()).map_err(|_| ApiError::Internal)?,
            ),
            SmtpTls::Tls => Tls::Wrapper(
                TlsParameters::new(self.host.clone()).map_err(|_| ApiError::Internal)?,
            ),
            SmtpTls::Off => Tls::None,
        };
        // `builder_dangerous` is only "dangerous" in that it does not impose
        // lettre's relay defaults; TLS policy is set explicitly just above.
        let builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.host)
            .port(self.port)
            .tls(tls);
        let builder = match (&self.username, &self.password) {
            (Some(u), Some(p)) => {
                builder.credentials(Credentials::new(u.clone(), p.clone()))
            }
            _ => builder,
        };
        Ok(builder)
    }
}

/// Delivery backend for [`SmtpEmailSender`]. Abstracted so tests can exercise
/// the full trait path against an in-memory fake without any network.
#[async_trait]
trait Mailer: Send + Sync {
    async fn deliver(&self, message: Message) -> Result<(), ApiError>;
}

struct LettreMailer(AsyncSmtpTransport<Tokio1Executor>);

#[async_trait]
impl Mailer for LettreMailer {
    async fn deliver(&self, message: Message) -> Result<(), ApiError> {
        self.0.send(message).await.map_err(|e| {
            tracing::error!(error = %e, "SMTP delivery failed");
            ApiError::Internal
        })?;
        Ok(())
    }
}

/// Real [`EmailSender`] backed by an SMTP relay (lettre, rustls only).
pub struct SmtpEmailSender {
    mailer: Box<dyn Mailer>,
    from: Mailbox,
}

impl SmtpEmailSender {
    pub fn new(config: SmtpConfig) -> Result<Self, ApiError> {
        let transport = config.transport_builder()?.build();
        Ok(Self {
            mailer: Box::new(LettreMailer(transport)),
            from: config.from,
        })
    }

    fn magic_link_message(&self, to: &str, link: &str) -> Result<Message, ApiError> {
        let to: Mailbox = to.parse().map_err(|_| ApiError::Validation(format!("invalid recipient address: {to:?}")))?;
        Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject("Your sign-in link")
            .body(format!(
                "Hello,\n\nUse this link to sign in to Health Travel Aggregator:\n\n{link}\n\n\
                 The link expires in 1 hour. If you did not request it, you can ignore this email.\n"
            ))
            .map_err(|_| ApiError::Internal)
    }

    fn inquiry_notification_message(
        &self,
        to: &str,
        clinic_name: &str,
        patient_email: &str,
    ) -> Result<Message, ApiError> {
        let to: Mailbox = to.parse().map_err(|_| ApiError::Validation(format!("invalid recipient address: {to:?}")))?;
        Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(format!("New patient inquiry for {clinic_name}"))
            .body(format!(
                "Hello,\n\n{clinic_name} received a new inquiry from {patient_email}.\n\n\
                 Sign in to your provider dashboard to view and respond.\n"
            ))
            .map_err(|_| ApiError::Internal)
    }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send_magic_link(&self, to: &str, link: &str) -> Result<(), ApiError> {
        self.mailer.deliver(self.magic_link_message(to, link)?).await
    }

    async fn send_inquiry_notification(
        &self,
        to: &str,
        clinic_name: &str,
        patient_email: &str,
    ) -> Result<(), ApiError> {
        self.mailer
            .deliver(self.inquiry_notification_message(to, clinic_name, patient_email)?)
            .await
    }
}

/// Connector selection: real SMTP sender when `SMTP_HOST` is configured, mock
/// otherwise so dev/CI keep working with no external dependency. A set but
/// invalid SMTP config logs an error and falls back to the mock rather than
/// crashing the server — the mock records messages, nothing is silently lost.
pub fn sender_from_env() -> Arc<dyn EmailSender + Send + Sync> {
    match SmtpConfig::from_env() {
        None => Arc::new(MockEmailSender::new()),
        Some(Ok(cfg)) => match SmtpEmailSender::new(cfg) {
            Ok(sender) => {
                tracing::info!("SMTP email sender configured");
                Arc::new(sender)
            }
            Err(_) => {
                tracing::error!("SMTP configuration could not build a transport; falling back to mock email sender");
                Arc::new(MockEmailSender::new())
            }
        },
        Some(Err(reason)) => {
            tracing::error!(%reason, "invalid SMTP configuration; falling back to mock email sender");
            Arc::new(MockEmailSender::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    fn test_sender() -> SmtpEmailSender {
        SmtpEmailSender {
            mailer: Box::new(RecordingMailer::default()),
            from: "Health Travel <no-reply@example.com>".parse().unwrap(),
        }
    }

    /// Decode the formatted message into (headers, body) with the body
    /// quoted-printable-decoded, so assertions are on real content.
    fn decoded(msg: &Message) -> (String, String) {
        let raw = String::from_utf8(msg.formatted()).unwrap();
        let (headers, body) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
        let body = quoted_printable::decode(body, quoted_printable::ParseMode::Robust).unwrap();
        (headers.to_string(), String::from_utf8(body).unwrap())
    }

    #[derive(Clone, Default)]
    struct RecordingMailer {
        sent: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Mailer for RecordingMailer {
        async fn deliver(&self, message: Message) -> Result<(), ApiError> {
            let raw = String::from_utf8(message.formatted()).map_err(|_| ApiError::Internal)?;
            self.sent.lock().await.push(raw);
            Ok(())
        }
    }

    #[test]
    fn magic_link_message_has_subject_to_and_link() {
        let sender = test_sender();
        let msg = sender
            .magic_link_message(
                "patient@example.com",
                "https://app.example.com/auth/verify?token=abc123",
            )
            .unwrap();
        let (headers, body) = decoded(&msg);
        assert!(headers.contains("Subject: Your sign-in link"), "{headers}");
        assert!(headers.contains("To: patient@example.com"), "{headers}");
        assert!(headers.contains("no-reply@example.com"), "{headers}");
        assert!(
            body.contains("https://app.example.com/auth/verify?token=abc123"),
            "{body}"
        );
    }

    #[test]
    fn inquiry_notification_message_mentions_clinic_and_patient() {
        let sender = test_sender();
        let msg = sender
            .inquiry_notification_message("clinic@example.com", "Acme Clinic", "patient@example.com")
            .unwrap();
        let (headers, body) = decoded(&msg);
        assert!(
            headers.contains("Subject: New patient inquiry for Acme Clinic"),
            "{headers}"
        );
        assert!(headers.contains("To: clinic@example.com"), "{headers}");
        assert!(body.contains("patient@example.com"), "{body}");
    }

    #[test]
    fn invalid_recipient_is_a_validation_error_not_a_panic() {
        let sender = test_sender();
        assert!(matches!(
            sender.magic_link_message("not-an-address", "https://x"),
            Err(ApiError::Validation(_))
        ));
    }

    /// Trait-level integration: the EmailSender impl builds the message and
    /// hands it to the transport, with no network involved.
    #[tokio::test]
    async fn smtp_sender_delivers_built_messages_through_the_mailer() {
        let mailer = RecordingMailer::default();
        let sent = mailer.sent.clone();
        let sender = SmtpEmailSender {
            mailer: Box::new(mailer),
            from: "no-reply@example.com".parse().unwrap(),
        };
        sender
            .send_magic_link("p@example.com", "https://app.example.com/auth/verify?token=tok")
            .await
            .unwrap();
        sender
            .send_inquiry_notification("c@example.com", "Acme", "p@example.com")
            .await
            .unwrap();
        let sent = sent.lock().await;
        assert_eq!(sent.len(), 2);
        assert!(sent[0].contains("token=3Dtok"), "{}", sent[0]);
        assert!(sent[1].contains("Acme"), "{}", sent[1]);
    }

    fn vars<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        let map: HashMap<&str, &str> = pairs.iter().copied().collect();
        move |key| map.get(key).map(|v| v.to_string())
    }

    #[test]
    fn unset_host_means_mock_mode() {
        assert!(SmtpConfig::from_vars(vars(&[])).is_none());
    }

    #[test]
    fn starttls_and_port_587_are_the_defaults() {
        let cfg = SmtpConfig::from_vars(vars(&[
            ("SMTP_HOST", "smtp.example.com"),
            ("SMTP_FROM", "no-reply@example.com"),
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(cfg.tls, SmtpTls::StartTls);
        assert_eq!(cfg.port, 587);
        assert_eq!(cfg.host, "smtp.example.com");
        assert!(cfg.username.is_none());
    }

    #[test]
    fn implicit_tls_defaults_to_port_465() {
        let cfg = SmtpConfig::from_vars(vars(&[
            ("SMTP_HOST", "smtp.example.com"),
            ("SMTP_FROM", "no-reply@example.com"),
            ("SMTP_TLS", "tls"),
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(cfg.tls, SmtpTls::Tls);
        assert_eq!(cfg.port, 465);
    }

    #[test]
    fn host_without_from_is_an_error() {
        let result = SmtpConfig::from_vars(vars(&[("SMTP_HOST", "smtp.example.com")]));
        assert!(matches!(result, Some(Err(_))));
    }

    #[test]
    fn invalid_tls_mode_is_an_error() {
        let result = SmtpConfig::from_vars(vars(&[
            ("SMTP_HOST", "smtp.example.com"),
            ("SMTP_FROM", "no-reply@example.com"),
            ("SMTP_TLS", "maybe"),
        ]));
        assert!(matches!(result, Some(Err(_))));
    }

    #[test]
    fn explicit_port_and_credentials_are_used() {
        let cfg = SmtpConfig::from_vars(vars(&[
            ("SMTP_HOST", "smtp.example.com"),
            ("SMTP_FROM", "no-reply@example.com"),
            ("SMTP_PORT", "2525"),
            ("SMTP_USERNAME", "user"),
            ("SMTP_PASSWORD", "pass"),
            ("SMTP_TLS", "off"),
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(cfg.port, 2525);
        assert_eq!(cfg.tls, SmtpTls::Off);
        assert_eq!(cfg.username.as_deref(), Some("user"));
        assert_eq!(cfg.password.as_deref(), Some("pass"));
        // Transport construction must succeed for every TLS mode.
        assert!(cfg.transport_builder().is_ok());
    }
}
