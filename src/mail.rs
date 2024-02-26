use std::fmt::Display;

#[cfg(feature = "mail")]
use secrecy::{ExposeSecret, Secret};
use tracing_log::log;

use crate::config::mail;
use crate::domain::{Subscriber, SubscriberEmail};

#[derive(Debug, Clone)]
pub struct Client {
    #[cfg(feature = "mail")]
    auth_token: Secret<String>,
    #[cfg(feature = "mail")]
    base_url: String,
    #[cfg(feature = "mail")]
    http_client: reqwest::Client,
    sender: SubscriberEmail,
}

impl Client {
    pub fn new(config: mail::Config) -> Result<Self, String> {
        #[cfg(feature = "mail")]
        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap();
        let sender = config.sender()?;
        Ok(Self {
            #[cfg(feature = "mail")]
            auth_token: config.auth_token,
            #[cfg(feature = "mail")]
            base_url: config.base_url,
            #[cfg(feature = "mail")]
            http_client,
            sender,
        })
    }

    pub async fn send(
        &self,
        recipient: &Subscriber,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> reqwest::Result<()> {
        let body = EmailRequest {
            from: self.sender.as_ref(),
            to: recipient.email.as_ref(),
            subject,
            html_body,
            text_body,
        };

        #[cfg(feature = "mail")]
        {
            log::trace!("Sending email: {}", body);
            let url = format!("{}/email", self.base_url);
            self.http_client
                .post(&url)
                .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
                .json(&body)
                .send()
                .await?
                .error_for_status()?;
        }

        #[cfg(not(feature = "mail"))]
        {
            log::info!("Sending email: {}", body);
        }

        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct EmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

impl Display for EmailRequest<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EmailRequest:
            From:    {}
            To:      {}
            Subject: {}
            HTML Body:
            {}
            Text Body:
            {}
            ",
            self.from, self.to, self.subject, self.html_body, self.text_body
        )
    }
}

#[cfg(test)]
#[cfg(feature = "mail")]
mod tests {
    use std::time::Duration;

    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::faker::name::en::Name;
    use fake::{Fake, Faker};
    use reqwest::{Method, StatusCode};
    use secrecy::Secret;
    use serde_json::{from_slice, Value};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use crate::config::mail::Config;
    use crate::domain::{Subscriber, SubscriberEmail, SubscriberName};

    use super::*;

    struct EmailRequestMatcher;

    impl wiremock::Match for EmailRequestMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<Value, _> = from_slice(&request.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_makes_expected_request() {
        let mock_server = MockServer::start().await;

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method(Method::POST))
            .and(EmailRequestMatcher)
            .respond_with(ResponseTemplate::new(StatusCode::OK))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = send(mock_server.uri()).await;

        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_fails_on_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = send(mock_server.uri()).await;

        assert_err!(result);
    }

    #[tokio::test]
    async fn send_times_out_if_waiting_too_long() {
        let mock_server = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(StatusCode::OK).set_delay(Duration::from_secs(180)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let result = send(mock_server.uri()).await;

        assert_err!(result);
    }

    pub async fn send(base_url: String) -> reqwest::Result<()> {
        let auth_token = Secret::new(Faker.fake());
        let sender = SafeEmail().fake();
        let timeout = Duration::from_millis(200);
        let mail_config = Config {
            auth_token,
            base_url,
            sender,
            timeout,
        };
        let mail_client = Client::new(mail_config).unwrap();

        let subscriber = Subscriber {
            email: SubscriberEmail::parse(SafeEmail().fake()).unwrap(),
            name: SubscriberName::parse(Name().fake()).unwrap(),
        };
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        mail_client
            .send(&subscriber, &subject, &content, &content)
            .await
    }
}
