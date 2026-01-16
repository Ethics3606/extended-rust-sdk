//! HTTP client for Extended Exchange REST API.

use reqwest::{header, Client, Method, Response};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;

use crate::config::EndpointConfig;
use crate::error::{ApiErrorResponse, ExtendedError, Result};

/// HTTP client for making requests to the Extended Exchange API.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    config: EndpointConfig,
    api_key: Option<String>,
}

impl HttpClient {
    /// Create a new HTTP client with the given configuration.
    pub fn new(config: EndpointConfig) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("extended-rust-sdk/0.1.0"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            config,
            api_key: None,
        })
    }

    /// Create a new HTTP client with API key authentication.
    pub fn with_api_key(config: EndpointConfig, api_key: impl Into<String>) -> Result<Self> {
        let mut client = Self::new(config)?;
        client.api_key = Some(api_key.into());
        Ok(client)
    }

    /// Get the endpoint configuration.
    pub fn config(&self) -> &EndpointConfig {
        &self.config
    }

    /// Make a GET request.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(Method::GET, path, Option::<&()>::None).await
    }

    /// Make a GET request with query parameters.
    pub async fn get_with_query<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let base_url = self.config.api_url(path);
        let url = self.build_url_with_query(&base_url, query)?;

        let mut request = self.client.get(url);

        if let Some(ref api_key) = self.api_key {
            request = request.header("X-Api-Key", api_key);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Make a POST request.
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.request(Method::POST, path, Some(body)).await
    }

    /// Make a POST request without a body.
    pub async fn post_empty<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(Method::POST, path, Option::<&()>::None).await
    }

    /// Make a PATCH request.
    pub async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.request(Method::PATCH, path, Some(body)).await
    }

    /// Make a DELETE request.
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(Method::DELETE, path, Option::<&()>::None).await
    }

    /// Make a DELETE request with query parameters.
    pub async fn delete_with_query<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let base_url = self.config.api_url(path);
        let url = self.build_url_with_query(&base_url, query)?;

        let mut request = self.client.delete(url);

        if let Some(ref api_key) = self.api_key {
            request = request.header("X-Api-Key", api_key);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Build a URL with query parameters.
    fn build_url_with_query<Q: Serialize>(&self, base_url: &str, query: &Q) -> Result<Url> {
        let mut url = Url::parse(base_url)?;

        // Serialize query to a map and add as query parameters
        let query_string = serde_urlencoded::to_string(query)
            .map_err(|e| ExtendedError::Serialization(serde_json::Error::io(
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            )))?;

        if !query_string.is_empty() {
            url.set_query(Some(&query_string));
        }

        Ok(url)
    }

    /// Internal method to make a request with optional body.
    async fn request<T: DeserializeOwned, B: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T> {
        let url = self.config.api_url(path);
        let mut request = self.client.request(method, &url);

        if let Some(ref api_key) = self.api_key {
            request = request.header("X-Api-Key", api_key);
        }

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Handle the API response, checking for errors.
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            // Get text first for better error messages
            let text = response.text().await?;
            match serde_json::from_str::<T>(&text) {
                Ok(body) => Ok(body),
                Err(e) => {
                    // Include part of the response in the error for debugging
                    let preview = if text.len() > 500 {
                        format!("{}...", &text[..500])
                    } else {
                        text
                    };
                    Err(ExtendedError::Serialization(serde_json::Error::io(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Failed to parse response: {}. Response: {}", e, preview)
                        )
                    )))
                }
            }
        } else if status.as_u16() == 429 {
            Err(ExtendedError::RateLimitExceeded)
        } else {
            // Try to parse as API error response
            let text = response.text().await?;
            match serde_json::from_str::<ApiErrorResponse>(&text) {
                Ok(error_resp) => Err(ExtendedError::from(error_resp)),
                Err(_) => Err(ExtendedError::Api {
                    code: status.as_u16().to_string(),
                    message: if text.is_empty() {
                        "(no response body)".to_string()
                    } else {
                        text
                    },
                }),
            }
        }
    }
}


