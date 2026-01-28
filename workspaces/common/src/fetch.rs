use anyhow::{Result, bail};
use gtk::gio::{self};
use std::time::Duration;
use tracing::{debug, error};
use ureq::Agent;

pub struct Response<T> {
    pub data: T,
    pub mimetype: Option<String>,
}

pub struct Fetch {
    agent: Agent,
}
impl Fetch {
    const FETCH_TIMEOUT: u64 = 5; // Seconds

    pub fn new() -> Self {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(Self::FETCH_TIMEOUT)))
            .user_agent("Wget/1.21.3")
            .build()
            .into();

        Self { agent }
    }

    pub async fn get_as_string(&self, url: &str) -> Result<Response<String>> {
        debug!("Fetching text from url: {url}");
        let agent_clone = self.agent.clone();
        let url = url.to_string();
        let url_clone = url.clone();

        match gio::spawn_blocking(move || -> Result<(String, Option<String>)> {
            let mut call = agent_clone.get(url_clone).call()?;
            let body = call.body_mut();
            let mimetype = body.mime_type().map(std::string::ToString::to_string);
            let response = body.read_to_string()?;
            Ok((response, mimetype))
        })
        .await
        {
            Ok(Ok((text, mimetype))) => Ok(Response {
                data: text,
                mimetype,
            }),
            Ok(Err(error)) => Self::error_handler(&url, &error),
            Err(error) => Self::error_handler(&url, &error),
        }
    }

    pub async fn get_as_bytes(&self, url: &str) -> Result<Response<Vec<u8>>> {
        debug!("Fetching bytes from url: {url}");
        let agent_clone = self.agent.clone();
        let url = url.to_string();
        let url_clone = url.clone();

        match gio::spawn_blocking(move || -> Result<(Vec<u8>, Option<String>)> {
            let mut call = agent_clone.get(url_clone).call()?;
            let body = call.body_mut();
            let mimetype = body.mime_type().map(std::string::ToString::to_string);
            let response = body.read_to_vec()?;
            Ok((response, mimetype))
        })
        .await
        {
            Ok(Ok((bytes, mimetype))) => Ok(Response {
                data: bytes,
                mimetype,
            }),
            Ok(Err(error)) => Self::error_handler(&url, &error),
            Err(error) => Self::error_handler(&url, &error),
        }
    }

    // Any error logged and a anyhow::Error
    fn error_handler<R>(url: &str, error: impl std::fmt::Debug) -> Result<R> {
        let message = format!("Fetching '{url}' failed: '{error:?}'");
        error!(message);
        bail!(message)
    }
}
