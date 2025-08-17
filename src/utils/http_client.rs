#![allow(dead_code)]

use reqwest::{
    Body, Client, ClientBuilder, Error as RequestError, Response, StatusCode,
    header::{HeaderMap, HeaderName, HeaderValue},
};

use bytes::Bytes;
use futures_util::Stream;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::option::Option;
use std::pin::Pin;
use std::time::Duration;

#[derive(Debug)]
pub enum HttpClientError {
    BuildError(RequestError),
    RequestError(RequestError),
    IoError(IoError),
}
impl fmt::Display for HttpClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpClientError::BuildError(err) => write!(f, "Failed to build client: {}", err),
            HttpClientError::RequestError(err) => write!(f, "Request failed: {}", err),
            HttpClientError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl From<IoError> for HttpClientError {
    fn from(err: IoError) -> Self {
        HttpClientError::IoError(err)
    }
}

impl Error for HttpClientError {}

pub struct HttpClientBuilder {
    client_builder: ClientBuilder,
    headers: HeaderMap,
    user_agent: Option<String>,
    timeout: Option<Duration>,
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        HttpClientBuilder {
            client_builder: ClientBuilder::new(),
            headers: HeaderMap::new(),
            user_agent: None,
            timeout: None,
        }
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn header(mut self, key: impl Into<HeaderName>, value: impl Into<HeaderValue>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        for (key, value) in headers {
            if let Ok(header_name) = key.parse::<HeaderName>() {
                if let Ok(header_value) = value.parse::<HeaderValue>() {
                    self.headers.insert(header_name, header_value);
                }
            }
        }
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<HttpClient, HttpClientError> {
        let mut client_builder = self.client_builder;
        if let Some(user_agent) = self.user_agent {
            client_builder = client_builder.user_agent(user_agent);
        }

        if let Some(timeout) = self.timeout {
            client_builder = client_builder.timeout(timeout);
        }

        client_builder = client_builder.default_headers(self.headers);

        match client_builder.build() {
            Ok(client) => Ok(HttpClient { client }),
            Err(e) => Err(HttpClientError::BuildError(e)),
        }
    }
}

pub struct HttpClientResponse {
    response: Response,
}

impl HttpClientResponse {
    pub fn new(response: Response) -> Self {
        HttpClientResponse { response }
    }

    pub async fn text(self) -> Result<String, HttpClientError> {
        self.response
            .text()
            .await
            .map_err(HttpClientError::RequestError)
    }

    pub async fn json<T: DeserializeOwned>(self) -> Result<T, HttpClientError> {
        self.response
            .json::<T>()
            .await
            .map_err(HttpClientError::RequestError)
    }

    pub async fn chunk(mut self) -> Result<Option<bytes::Bytes>, HttpClientError> {
        self.response
            .chunk()
            .await
            .map_err(HttpClientError::RequestError)
    }

    pub async fn bytes(self) -> Result<bytes::Bytes, HttpClientError> {
        self.response
            .bytes()
            .await
            .map_err(HttpClientError::RequestError)
    }

    pub fn bytes_stream(self) -> Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>>>> {
        Box::pin(self.response.bytes_stream())
    }

    pub fn status_code(&self) -> StatusCode {
        self.response.status()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.response.headers()
    }
}

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub async fn get(&self, url: &str) -> Result<HttpClientResponse, HttpClientError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(HttpClientError::RequestError)?;
        Ok(HttpClientResponse::new(response))
    }

    pub async fn post(
        &self,
        url: &str,
        body: impl Into<Body>,
    ) -> Result<HttpClientResponse, HttpClientError> {
        let response = self
            .client
            .post(url)
            .body(body.into())
            .send()
            .await
            .map_err(HttpClientError::RequestError)?;
        Ok(HttpClientResponse::new(response))
    }
}
