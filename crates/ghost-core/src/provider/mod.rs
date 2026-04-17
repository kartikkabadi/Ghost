//! Provider abstractions and KAR-160: Grok 4.20 model mapping.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Provider kinds & capabilities ───────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderKind {
    OpenRouter,
    Xai,
    Firecrawl,
    Exa,
}

impl ProviderKind {
    pub fn slug(&self) -> &'static str {
        match self {
            Self::OpenRouter => "openRouter",
            Self::Xai => "xai",
            Self::Firecrawl => "firecrawl",
            Self::Exa => "exa",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderCapability {
    Plan,
    Reason,
    Search,
    Crawl,
    Extract,
    Enrich,
    Report,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderTarget {
    pub kind: ProviderKind,
    pub base_url: String,
    pub api_key_env: String,
    pub default_operation: String,
    pub capabilities: Vec<ProviderCapability>,
}

impl ProviderTarget {
    pub fn new(
        kind: ProviderKind,
        base_url: impl Into<String>,
        api_key_env: impl Into<String>,
        default_operation: impl Into<String>,
        capabilities: Vec<ProviderCapability>,
    ) -> Self {
        Self {
            kind,
            base_url: base_url.into(),
            api_key_env: api_key_env.into(),
            default_operation: default_operation.into(),
            capabilities,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderRuntime {
    pub targets: HashMap<ProviderKind, ProviderTarget>,
}

impl ProviderRuntime {
    pub fn new(targets: HashMap<ProviderKind, ProviderTarget>) -> Self {
        Self { targets }
    }

    pub fn default_targets() -> HashMap<ProviderKind, ProviderTarget> {
        HashMap::from([
            (
                ProviderKind::OpenRouter,
                ProviderTarget::new(
                    ProviderKind::OpenRouter,
                    "https://openrouter.ai",
                    "OPENROUTER_API_KEY",
                    "chat/completions",
                    vec![ProviderCapability::Plan, ProviderCapability::Reason, ProviderCapability::Report],
                ),
            ),
            (
                ProviderKind::Xai,
                ProviderTarget::new(
                    ProviderKind::Xai,
                    "https://api.x.ai",
                    "XAI_API_KEY",
                    "chat/completions",
                    vec![ProviderCapability::Plan, ProviderCapability::Reason, ProviderCapability::Report],
                ),
            ),
            (
                ProviderKind::Firecrawl,
                ProviderTarget::new(
                    ProviderKind::Firecrawl,
                    "https://api.firecrawl.dev",
                    "FIRECRAWL_API_KEY",
                    "crawl",
                    vec![ProviderCapability::Crawl, ProviderCapability::Extract, ProviderCapability::Search],
                ),
            ),
            (
                ProviderKind::Exa,
                ProviderTarget::new(
                    ProviderKind::Exa,
                    "https://api.exa.ai",
                    "EXA_API_KEY",
                    "search",
                    vec![ProviderCapability::Search, ProviderCapability::Enrich],
                ),
            ),
        ])
    }

    pub fn target(&self, kind: &ProviderKind) -> Option<&ProviderTarget> {
        self.targets.get(kind)
    }

    pub fn register(&mut self, target: ProviderTarget) {
        self.targets.insert(target.kind.clone(), target);
    }
}

impl Default for ProviderRuntime {
    fn default() -> Self {
        Self {
            targets: Self::default_targets(),
        }
    }
}

// ─── KAR-160: Grok 4.20 model mapping ────────────────────────────────────────

/// SDK dispatch pattern for a xAI model request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Grok420SdkPattern {
    /// Standard OpenAI-compatible `chat/completions` endpoint.
    ChatCompletions,
    /// xAI multi-agent `responses` API – carries `agent_count` and
    /// `reasoning_effort` in the request body.
    MultiAgent,
}

/// Canonical information resolved from any Grok 4.20 model ID alias.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grok420ModelId {
    /// The canonical, unambiguous xAI model identifier to send in the request.
    pub canonical_id: String,
    /// Which SDK dispatch pattern to use when calling this model.
    pub sdk_pattern: Grok420SdkPattern,
    /// Recommended agent count for multi-agent dispatch (4 = focused, 16 = deep).
    pub agent_count: Option<u8>,
    /// Whether to enable extended reasoning for this model variant.
    pub reasoning_effort: Option<String>,
}

/// Maps all known Grok 4.20 alias strings to a canonical `Grok420ModelId`.
/// Addresses KAR-160: Grok 4.20 mapping.
pub struct Grok420ModelMapper;

impl Grok420ModelMapper {
    /// Resolve an arbitrary Grok 4.20 model alias (user-supplied or from config)
    /// to its canonical form.
    ///
    /// Returns `None` if the alias is not a recognised Grok 4.20 variant.
    pub fn resolve(model_id: &str) -> Option<Grok420ModelId> {
        let normalised = model_id.trim().to_lowercase();
        let normalised = normalised
            .replace("grok-420", "grok-4.20")  // normalise dash-only form
            .replace("grok420", "grok-4.20");   // normalise compact form

        match normalised.as_str() {
            // ── Standard reasoning variants ──────────────────────────────────
            "grok-4.20"
            | "grok-4.20-0309"
            | "grok-4.20-0309-reasoning"
            | "grok-4.20-reasoning" => Some(Grok420ModelId {
                canonical_id: "grok-4.20-0309-reasoning".to_string(),
                sdk_pattern: Grok420SdkPattern::ChatCompletions,
                agent_count: None,
                reasoning_effort: Some("high".to_string()),
            }),

            // ── Fast / low-latency variant ───────────────────────────────────
            "grok-4.20-fast" | "grok-4.20-fast-0309" => Some(Grok420ModelId {
                canonical_id: "grok-4.20-fast".to_string(),
                sdk_pattern: Grok420SdkPattern::ChatCompletions,
                agent_count: None,
                reasoning_effort: Some("low".to_string()),
            }),

            // ── Multi-agent / beta variants ──────────────────────────────────
            "grok-4.20-multi-agent"
            | "grok-4.20-beta"
            | "grok-4.20-multi-agent-beta" => Some(Grok420ModelId {
                canonical_id: "grok-4.20-multi-agent".to_string(),
                sdk_pattern: Grok420SdkPattern::MultiAgent,
                agent_count: Some(4),
                reasoning_effort: Some("medium".to_string()),
            }),

            _ => None,
        }
    }

    /// Build the `model` and `agent_count` / `reasoning_effort` fields for the
    /// xAI API request body, given a resolved `Grok420ModelId`.
    pub fn dispatch_request(
        model: &Grok420ModelId,
        base_body: serde_json::Value,
    ) -> serde_json::Value {
        let mut body = match base_body {
            serde_json::Value::Object(m) => m,
            _ => serde_json::Map::new(),
        };

        body.insert(
            "model".to_string(),
            serde_json::Value::String(model.canonical_id.clone()),
        );

        if model.sdk_pattern == Grok420SdkPattern::MultiAgent {
            if let Some(count) = model.agent_count {
                body.insert(
                    "agent_count".to_string(),
                    serde_json::Value::Number(count.into()),
                );
            }
        }

        if let Some(effort) = &model.reasoning_effort {
            body.insert(
                "reasoning_effort".to_string(),
                serde_json::Value::String(effort.clone()),
            );
        }

        serde_json::Value::Object(body)
    }

    /// Return `true` if `model_id` is any recognised Grok 4.20 alias.
    pub fn is_grok_420(model_id: &str) -> bool {
        Self::resolve(model_id).is_some()
    }
}
