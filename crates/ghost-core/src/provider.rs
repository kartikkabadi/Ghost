use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
