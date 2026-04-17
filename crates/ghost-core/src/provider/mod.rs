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
#[serde(rename_all = "camelCase")]
pub struct ModelProfile {
    pub provider: ProviderKind,
    pub model: String,
    pub base_url: String,
    pub sdk_pattern: String,
    pub agent_modes: Vec<u8>,
    pub specialization: Option<String>,
    pub notes: Vec<String>,
}

impl ModelProfile {
    pub fn new(
        provider: ProviderKind,
        model: impl Into<String>,
        base_url: impl Into<String>,
        sdk_pattern: impl Into<String>,
        agent_modes: Vec<u8>,
        specialization: Option<String>,
        notes: Vec<String>,
    ) -> Self {
        Self {
            provider,
            model: model.into(),
            base_url: base_url.into(),
            sdk_pattern: sdk_pattern.into(),
            agent_modes,
            specialization,
            notes,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderRuntime {
    pub targets: HashMap<ProviderKind, ProviderTarget>,
    pub models: Vec<ModelProfile>,
}

impl ProviderRuntime {
    pub fn new(targets: HashMap<ProviderKind, ProviderTarget>) -> Self {
        Self {
            targets,
            models: Self::default_models(),
        }
    }

    pub fn default_models() -> Vec<ModelProfile> {
        vec![
            ModelProfile::new(
                ProviderKind::Xai,
                "Grok 4.20 Multi-Agent beta",
                "https://api.x.ai/v1",
                "multi-agent",
                vec![4, 16],
                Some("scheduler".to_string()),
                vec![
                    "Choose for wave planning".to_string(),
                    "Use for 4-agent and 16-agent execution modes".to_string(),
                ],
            ),
            ModelProfile::new(
                ProviderKind::Xai,
                "Grok 4.20",
                "https://api.x.ai/v1",
                "chat-completions",
                vec![4],
                Some("reasoner".to_string()),
                vec!["Fallback synthesis profile".to_string()],
            ),
        ]
    }

    pub fn default_targets() -> HashMap<ProviderKind, ProviderTarget> {
        HashMap::from([
            (
                ProviderKind::OpenRouter,
                ProviderTarget::new(
                    ProviderKind::OpenRouter,
                    "https://openrouter.ai/api/v1",
                    "OPENROUTER_API_KEY",
                    "chat/completions",
                    vec![ProviderCapability::Plan, ProviderCapability::Reason, ProviderCapability::Report],
                ),
            ),
            (
                ProviderKind::Xai,
                ProviderTarget::new(
                    ProviderKind::Xai,
                    "https://api.x.ai/v1",
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

    pub fn model_profiles(&self) -> &[ModelProfile] {
        &self.models
    }

    pub fn grok_420_profile(&self) -> Option<&ModelProfile> {
        self.models.iter().find(|profile| profile.model == "Grok 4.20 Multi-Agent beta")
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
            models: Self::default_models(),
        }
    }
}
