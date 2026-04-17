use crate::provider::{ProviderCapability, ProviderKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider: ProviderKind,
    pub capability: ProviderCapability,
    pub tags: Vec<String>,
    pub input_keys: Vec<String>,
    pub output_keys: Vec<String>,
    pub parallelizable: bool,
}

impl SkillSpec {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        provider: ProviderKind,
        capability: ProviderCapability,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            provider,
            capability,
            tags: Vec::new(),
            input_keys: Vec::new(),
            output_keys: Vec::new(),
            parallelizable: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillRegistry {
    pub skills: HashMap<String, SkillSpec>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, spec: SkillSpec) {
        self.skills.insert(spec.id.clone(), spec);
    }

    pub fn get(&self, id: &str) -> Option<&SkillSpec> {
        self.skills.get(id)
    }

    pub fn by_capability(&self, capability: ProviderCapability) -> Vec<&SkillSpec> {
        self.skills.values().filter(|skill| skill.capability == capability).collect()
    }

    pub fn ordered_parallel_skills(&self) -> Vec<&SkillSpec> {
        let mut skills: Vec<&SkillSpec> = self.skills.values().filter(|skill| skill.parallelizable).collect();
        skills.sort_by(|a, b| a.id.cmp(&b.id));
        skills
    }

    pub fn defaults() -> Self {
        let mut registry = Self::new();
        registry.register(SkillSpec {
            id: "planner.openrouter".to_string(),
            name: "Planner".to_string(),
            description: "Builds the task plan and refines the objective into waves.".to_string(),
            provider: ProviderKind::OpenRouter,
            capability: ProviderCapability::Plan,
            tags: vec!["planning".to_string(), "orchestration".to_string()],
            input_keys: vec!["objective".to_string()],
            output_keys: vec!["plan".to_string()],
            parallelizable: false,
        });
        registry.register(SkillSpec {
            id: "reasoner.xai".to_string(),
            name: "Reasoner".to_string(),
            description: "Performs synthesis and analysis passes for the current wave.".to_string(),
            provider: ProviderKind::Xai,
            capability: ProviderCapability::Reason,
            tags: vec!["reasoning".to_string(), "synthesis".to_string()],
            input_keys: vec!["task".to_string()],
            output_keys: vec!["analysis".to_string()],
            parallelizable: true,
        });
        registry.register(SkillSpec {
            id: "verifier.xai".to_string(),
            name: "Verifier".to_string(),
            description: "Independently validates evidence-backed claims after worker tasks complete.".to_string(),
            provider: ProviderKind::Xai,
            capability: ProviderCapability::Reason,
            tags: vec!["verification".to_string(), "evidence".to_string()],
            input_keys: vec!["evidence".to_string()],
            output_keys: vec!["validated_claims".to_string()],
            parallelizable: false,
        });
        registry.register(SkillSpec {
            id: "crawler.firecrawl".to_string(),
            name: "Crawler".to_string(),
            description: "Fetches web pages and transforms them into structured evidence.".to_string(),
            provider: ProviderKind::Firecrawl,
            capability: ProviderCapability::Crawl,
            tags: vec!["crawl".to_string(), "fetch".to_string()],
            input_keys: vec!["url".to_string()],
            output_keys: vec!["document".to_string()],
            parallelizable: true,
        });
        registry.register(SkillSpec {
            id: "finder.exa".to_string(),
            name: "Finder".to_string(),
            description: "Searches the open web for candidate sources and anchors.".to_string(),
            provider: ProviderKind::Exa,
            capability: ProviderCapability::Search,
            tags: vec!["search".to_string(), "discovery".to_string()],
            input_keys: vec!["query".to_string()],
            output_keys: vec!["results".to_string()],
            parallelizable: true,
        });

        registry.register(SkillSpec {
            id: "recursive-search.exa".to_string(),
            name: "Recursive Search".to_string(),
            description: "Expands seeds through recursive search, dorking templates, and social footprint pivots".to_string(),
            provider: ProviderKind::Exa,
            capability: ProviderCapability::Search,
            tags: vec!["osint".to_string(), "recursive".to_string(), "dorking".to_string(), "pivot".to_string()],
            input_keys: vec!["seed_queries".to_string(), "pivot_rules".to_string(), "depth_limit".to_string()],
            output_keys: vec!["expanded_queries".to_string(), "candidate_sources".to_string()],
            parallelizable: true,
        });
        registry.register(SkillSpec {
            id: "metadata.cross-check".to_string(),
            name: "Metadata Cross Check".to_string(),
            description: "Cross verifies EXIF, docs, images, video, OCR, and timestamp or location mismatches".to_string(),
            provider: ProviderKind::Xai,
            capability: ProviderCapability::Enrich,
            tags: vec!["osint".to_string(), "metadata".to_string(), "exif".to_string(), "ocr".to_string()],
            input_keys: vec!["artifact_url".to_string(), "metadata_fields".to_string()],
            output_keys: vec!["mismatch_report".to_string(), "verified_fields".to_string()],
            parallelizable: true,
        });
        registry
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::defaults()
    }
}
