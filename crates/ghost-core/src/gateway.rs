use crate::provider::{ProviderKind, ProviderRuntime, ProviderTarget};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayRequest {
    pub provider: ProviderKind,
    pub operation: String,
    pub method: String,
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub body: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayResponse {
    pub provider: ProviderKind,
    pub operation: String,
    pub status: u16,
    pub body: Value,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatewayError {
    MissingProvider(ProviderKind),
    UnsupportedOperation { provider: ProviderKind, operation: String },
    MissingConfiguration(String),
}

impl Display for GatewayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingProvider(provider) => write!(f, "missing runtime configuration for provider {:?}", provider),
            Self::UnsupportedOperation { provider, operation } => {
                write!(f, "unsupported operation '{}' for provider {:?}", operation, provider)
            }
            Self::MissingConfiguration(message) => f.write_str(message),
        }
    }
}

impl Error for GatewayError {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ByokProviderGateway {
    runtime: ProviderRuntime,
}

impl ByokProviderGateway {
    pub fn new() -> Self {
        Self {
            runtime: ProviderRuntime::default(),
        }
    }

    pub fn with_runtime(runtime: ProviderRuntime) -> Self {
        Self { runtime }
    }

    pub fn runtime(&self) -> &ProviderRuntime {
        &self.runtime
    }

    pub fn register_target(&mut self, target: ProviderTarget) {
        self.runtime.register(target);
    }

    pub fn plan_request(
        &self,
        provider: ProviderKind,
        operation: impl Into<String>,
        body: Value,
    ) -> Result<GatewayRequest, GatewayError> {
        let operation = operation.into();
        let target = self.runtime.target(&provider).ok_or_else(|| GatewayError::MissingProvider(provider.clone()))?;
        let path = self.operation_path(&provider, &operation, target)?;
        let url = format!("{}/{}", target.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        let mut headers = BTreeMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("x-byok-provider".to_string(), provider.slug().to_string());

        Ok(GatewayRequest {
            provider,
            operation,
            method: "POST".to_string(),
            url,
            headers,
            body,
        })
    }

    pub fn normalize_response(
        &self,
        provider: ProviderKind,
        operation: impl Into<String>,
        raw: Value,
        status: u16,
    ) -> GatewayResponse {
        GatewayResponse {
            provider,
            operation: operation.into(),
            status,
            body: raw,
            request_id: None,
        }
    }

    pub fn default_request(&self, provider: ProviderKind, operation: impl Into<String>) -> Result<GatewayRequest, GatewayError> {
        self.plan_request(provider, operation, json!({}))
    }

    fn operation_path(
        &self,
        provider: &ProviderKind,
        operation: &str,
        target: &ProviderTarget,
    ) -> Result<String, GatewayError> {
        if operation.trim().is_empty() {
            return Err(GatewayError::MissingConfiguration("operation name cannot be empty".to_string()));
        }

        let normalized = operation.trim().replace('.', "/");
        let path = match provider {
            ProviderKind::OpenRouter => match normalized.as_str() {
                "chat/completions" | "chat" => target.default_operation.clone(),
                other => other.to_string(),
            },
            ProviderKind::Xai => match normalized.as_str() {
                "chat/completions" | "chat" => target.default_operation.clone(),
                other => other.to_string(),
            },
            ProviderKind::Firecrawl => match normalized.as_str() {
                "crawl" | "crawl/url" => target.default_operation.clone(),
                "extract" | "content/extract" => "extract".to_string(),
                other => other.to_string(),
            },
            ProviderKind::Exa => match normalized.as_str() {
                "search" | "search/query" => target.default_operation.clone(),
                other => other.to_string(),
            },
        };

        if path.is_empty() {
            return Err(GatewayError::UnsupportedOperation {
                provider: provider.clone(),
                operation: operation.to_string(),
            });
        }

        Ok(path)
    }
}

impl Default for ByokProviderGateway {
    fn default() -> Self {
        Self::new()
    }
}
