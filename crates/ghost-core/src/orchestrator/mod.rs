use crate::db::{EvidenceGraph, EvidenceRecord};
use crate::firecrawl::FirecrawlClient;
use crate::gateway::{ByokProviderGateway, GatewayRequest};
use crate::provider::{ProviderCapability, ProviderKind};
use crate::skills::{SkillRegistry, SkillSpec};
use crate::verification::{VerificationMesh, VerificationMeshReport};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::task::JoinSet;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    Plan,
    Search,
    Crawl,
    Extract,
    Enrich,
    Verify,
    Synthesize,
    Report,
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub kind: TaskKind,
    pub skill_id: Option<String>,
    pub provider: Option<ProviderKind>,
    pub input: Value,
    pub depends_on: Vec<String>,
}

impl Task {
    pub fn new(id: impl Into<String>, kind: TaskKind) -> Self {
        Self {
            id: id.into(),
            kind,
            skill_id: None,
            provider: None,
            input: Value::Null,
            depends_on: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskExecutionOutcome {
    pub task_id: String,
    pub kind: TaskKind,
    pub provider: Option<ProviderKind>,
    pub success: bool,
    pub summary: String,
    pub payload: Value,
    pub notes: Vec<String>,
    pub evidence: Vec<EvidenceRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationPlan {
    pub objective: String,
    pub tasks: Vec<Task>,
    pub gaps: Vec<String>,
    pub next_queries: Vec<String>,
}

impl OrchestrationPlan {
    pub fn from_objective(objective: impl Into<String>, registry: &SkillRegistry) -> Self {
        let objective = objective.into();
        let mut tasks = Vec::new();
        let mut search_task_ids = Vec::new();
        let mut content_task_ids = Vec::new();
        let mut analysis_task_ids = Vec::new();

        let planner = Task {
            id: "plan-1".to_string(),
            kind: TaskKind::Plan,
            skill_id: registry.get("planner.openrouter").map(|skill| skill.id.clone()),
            provider: Some(ProviderKind::OpenRouter),
            input: json!({"objective": objective.clone()}),
            depends_on: Vec::new(),
        };
        tasks.push(planner);

        for skill in registry.ordered_parallel_skills() {
            let capability = skill.capability.clone();
            if matches!(&capability, ProviderCapability::Plan) {
                continue;
            }
            if skill.id == "verifier.xai" {
                continue;
            }

            let task_id = format!("{}-task", skill.id.replace('.', "-"));
            let kind = match &capability {
                ProviderCapability::Search => TaskKind::Search,
                ProviderCapability::Crawl => TaskKind::Crawl,
                ProviderCapability::Extract => TaskKind::Extract,
                ProviderCapability::Enrich => TaskKind::Enrich,
                ProviderCapability::Reason | ProviderCapability::Report | ProviderCapability::Plan => TaskKind::Custom(skill.id.clone()),
            };

            let depends_on = match capability {
                ProviderCapability::Search => vec!["plan-1".to_string()],
                ProviderCapability::Crawl | ProviderCapability::Extract | ProviderCapability::Enrich => {
                    if search_task_ids.is_empty() {
                        vec!["plan-1".to_string()]
                    } else {
                        search_task_ids.clone()
                    }
                }
                ProviderCapability::Reason | ProviderCapability::Report | ProviderCapability::Plan => {
                    if !content_task_ids.is_empty() {
                        content_task_ids.clone()
                    } else if !search_task_ids.is_empty() {
                        search_task_ids.clone()
                    } else {
                        vec!["plan-1".to_string()]
                    }
                }
            };

            match capability {
                ProviderCapability::Search => search_task_ids.push(task_id.clone()),
                ProviderCapability::Crawl | ProviderCapability::Extract | ProviderCapability::Enrich => content_task_ids.push(task_id.clone()),
                ProviderCapability::Reason | ProviderCapability::Report => analysis_task_ids.push(task_id.clone()),
                ProviderCapability::Plan => {}
            }

            tasks.push(Task {
                id: task_id,
                kind,
                skill_id: Some(skill.id.clone()),
                provider: Some(skill.provider.clone()),
                input: json!({
                    "objective": objective.clone(),
                    "skill": skill.id.clone(),
                    "capability": format!("{:?}", &skill.capability),
                    "tags": skill.tags.clone(),
                }),
                depends_on,
            });
        }

        let mut verification_deps = Vec::new();
        verification_deps.extend(search_task_ids.clone());
        verification_deps.extend(content_task_ids.clone());
        verification_deps.extend(analysis_task_ids.clone());
        if verification_deps.is_empty() {
            verification_deps.push("plan-1".to_string());
        }

        let verifier_skill = registry.get("verifier.xai").or_else(|| registry.get("reasoner.xai"));
        let verifier = Task {
            id: "verify-1".to_string(),
            kind: TaskKind::Verify,
            skill_id: verifier_skill.map(|skill| skill.id.clone()),
            provider: Some(ProviderKind::Xai),
            input: json!({
                "objective": objective.clone(),
                "validationMode": "evidence_based",
            }),
            depends_on: verification_deps,
        };
        tasks.push(verifier);

        let synthesis = Task {
            id: "synthesis-1".to_string(),
            kind: TaskKind::Synthesize,
            skill_id: registry.get("reasoner.xai").map(|skill| skill.id.clone()),
            provider: Some(ProviderKind::Xai),
            input: json!({"objective": objective.clone()}),
            depends_on: vec!["verify-1".to_string()],
        };
        tasks.push(synthesis);

        let report = Task {
            id: "report-1".to_string(),
            kind: TaskKind::Report,
            skill_id: registry.get("reasoner.xai").map(|skill| skill.id.clone()),
            provider: Some(ProviderKind::OpenRouter),
            input: json!({"objective": objective.clone()}),
            depends_on: vec!["synthesis-1".to_string()],
        };
        tasks.push(report);

        let gaps = vec!["No live provider calls executed yet; outputs are provisional until connected credentials are supplied.".to_string()];
        let next_queries = vec![
            format!("Search for corroborating evidence about {}", objective),
            format!("Verify the strongest claims about {}", objective),
        ];

        Self {
            objective,
            tasks,
            gaps,
            next_queries,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestratorContract {
    pub objective: String,
    pub tasks: Vec<Task>,
    pub gaps: Vec<String>,
    pub next_queries: Vec<String>,
    pub max_waves: usize,
}

impl OrchestratorContract {
    pub fn from_objective(objective: impl Into<String>, registry: &SkillRegistry) -> Self {
        let plan = OrchestrationPlan::from_objective(objective, registry);
        Self {
            objective: plan.objective,
            tasks: plan.tasks,
            gaps: plan.gaps,
            next_queries: plan.next_queries,
            max_waves: 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorError {
    DependencyCycle { remaining: Vec<String> },
    EmptyPlan,
}

impl Display for OrchestratorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DependencyCycle { remaining } => write!(f, "dependency cycle detected for tasks: {:?}", remaining),
            Self::EmptyPlan => f.write_str("orchestration plan has no tasks"),
        }
    }
}

impl Error for OrchestratorError {}

pub type TaskFuture = Pin<Box<dyn Future<Output = TaskExecutionOutcome> + Send>>;
pub type TaskExecutor = Arc<dyn Fn(Task) -> TaskFuture + Send + Sync>;

#[derive(Clone, Debug, Default)]
pub struct WaveScheduler;

impl WaveScheduler {
    pub fn plan_waves(&self, tasks: &[Task]) -> Result<Vec<Vec<Task>>, OrchestratorError> {
        if tasks.is_empty() {
            return Err(OrchestratorError::EmptyPlan);
        }

        let mut remaining: HashMap<String, Task> = tasks.iter().cloned().map(|task| (task.id.clone(), task)).collect();
        let mut complete: HashSet<String> = HashSet::new();
        let mut waves = Vec::new();

        while !remaining.is_empty() {
            let ready_ids: Vec<String> = remaining
                .values()
                .filter(|task| task.depends_on.iter().all(|dependency| complete.contains(dependency)))
                .map(|task| task.id.clone())
                .collect();

            if ready_ids.is_empty() {
                return Err(OrchestratorError::DependencyCycle {
                    remaining: remaining.keys().cloned().collect(),
                });
            }

            let mut wave = Vec::new();
            for task_id in ready_ids {
                if let Some(task) = remaining.remove(&task_id) {
                    complete.insert(task.id.clone());
                    wave.push(task);
                }
            }
            waves.push(wave);
        }

        Ok(waves)
    }

    pub async fn execute(&self, tasks: Vec<Task>, executor: TaskExecutor) -> Result<Vec<TaskExecutionOutcome>, OrchestratorError> {
        let waves = self.plan_waves(&tasks)?;
        let mut results = Vec::new();

        for wave in waves {
            let mut join_set = JoinSet::new();
            for task in wave {
                let exec = executor.clone();
                join_set.spawn(async move { (exec)(task).await });
            }

            while let Some(join_result) = join_set.join_next().await {
                let outcome = join_result.expect("task execution panicked");
                results.push(outcome);
            }
        }

        results.sort_by(|a, b| a.task_id.cmp(&b.task_id));
        Ok(results)
    }
}

#[derive(Clone, Debug)]
pub struct TaskBroker {
    contract: OrchestratorContract,
    scheduler: WaveScheduler,
}

impl TaskBroker {
    pub fn new(contract: OrchestratorContract) -> Self {
        Self {
            contract,
            scheduler: WaveScheduler::default(),
        }
    }

    pub fn contract(&self) -> &OrchestratorContract {
        &self.contract
    }

    pub async fn execute_with(&self, executor: impl Fn(Task) -> TaskFuture + Send + Sync + 'static) -> Result<Vec<TaskExecutionOutcome>, OrchestratorError> {
        self.scheduler.execute(self.contract.tasks.clone(), Arc::new(executor)).await
    }
}

pub async fn simulate_task(
    gateway: &ByokProviderGateway,
    skills: &SkillRegistry,
    store: &EvidenceGraph,
    task: Task,
) -> TaskExecutionOutcome {
    let task_id = task.id.clone();
    let kind = task.kind.clone();
    let provider = task.provider.clone();
    let skill = task.skill_id.as_deref().and_then(|skill_id| skills.get(skill_id)).cloned();
    let objective = task.input.get("objective").and_then(Value::as_str).unwrap_or("unknown objective");
    let provider_name = provider.as_ref().map(|p| p.slug().to_string()).unwrap_or_else(|| "local".to_string());

    let request: Option<GatewayRequest> = provider
        .clone()
        .map(|provider_kind| {
            let operation = skill
                .as_ref()
                .map(skill_operation)
                .unwrap_or_else(|| task.kind_name());
            gateway.plan_request(provider_kind, operation, task.input.clone()).ok()
        })
        .flatten();

    let mut evidence = Vec::new();
    let mut notes = vec!["Executed by the wave scheduler".to_string()];
    let summary = match &kind {
        TaskKind::Plan => format!("Objective '{}' decomposed into a wave-based plan", objective),
        TaskKind::Search => {
            let discovered_links = gather_links(&task.input, store, objective).await;
            let claim = if discovered_links.is_empty() {
                format!("Search wave completed for {} but no candidate links were surfaced yet", objective)
            } else {
                format!("Search wave surfaced {} candidate link(s) for downstream extraction", discovered_links.len())
            };
            evidence.push(EvidenceRecord {
                id: format!("ev-{}", task_id),
                objective: objective.to_string(),
                task_id: task_id.clone(),
                claim: claim.clone(),
                source: if discovered_links.is_empty() { provider_name.clone() } else { discovered_links[0].clone() },
                provider: provider_name.clone(),
                confidence: if discovered_links.is_empty() { 0.45 } else { 0.7 },
                contradictions: Vec::new(),
                notes: vec![format!("{} discovered link(s)", discovered_links.len())],
                payload: json!({
                    "task": task.kind_name(),
                    "objective": objective,
                    "summary": claim,
                    "discoveredLinks": discovered_links,
                }),
                generated_at: current_timestamp(),
            });
            "Search candidates collected".to_string()
        }
        TaskKind::Crawl | TaskKind::Extract => {
            let discovered_links = gather_links(&task.input, store, objective).await;
            let client = FirecrawlClient::from_env();
            let pages = client.extract_pages(&discovered_links, objective).await;
            if client.is_configured() {
                notes.push("Firecrawl extraction used for discovered links".to_string());
            } else {
                notes.push("Firecrawl API key missing; local fallback extraction used".to_string());
            }
            if pages.is_empty() {
                evidence.push(EvidenceRecord {
                    id: format!("ev-{}", task_id),
                    objective: objective.to_string(),
                    task_id: task_id.clone(),
                    claim: format!("No discovered links were available for Firecrawl extraction on {}", objective),
                    source: provider_name.clone(),
                    provider: provider_name.clone(),
                    confidence: 0.3,
                    contradictions: Vec::new(),
                    notes: vec!["Extraction wave ran without links".to_string()],
                    payload: json!({
                        "task": task.kind_name(),
                        "objective": objective,
                        "summary": "No links to extract",
                        "discoveredLinks": discovered_links,
                        "firecrawlConfigured": client.is_configured(),
                    }),
                    generated_at: current_timestamp(),
                });
            } else {
                for page in pages {
                    let mut record_notes = vec!["Deep content extracted from discovered link".to_string()];
                    if page.fallback {
                        record_notes.push("Local fallback page synthesis used".to_string());
                    }
                    evidence.push(EvidenceRecord {
                        id: format!("ev-{}-{}", task_id, slugify(&page.url)),
                        objective: objective.to_string(),
                        task_id: task_id.clone(),
                        claim: if !page.summary.is_empty() {
                            page.summary.clone()
                        } else if let Some(first) = page.key_points.first() {
                            first.clone()
                        } else {
                            format!("Extracted page at {}", page.url)
                        },
                        source: page.url.clone(),
                        provider: "firecrawl".to_string(),
                        confidence: if page.fallback { 0.58 } else { 0.9 },
                        contradictions: Vec::new(),
                        notes: record_notes,
                        payload: json!({
                            "task": task.kind_name(),
                            "objective": objective,
                            "firecrawl": true,
                            "title": page.title,
                            "url": page.url,
                            "summary": page.summary,
                            "markdown": page.markdown,
                            "keyPoints": page.key_points,
                            "extractedClaims": page.extracted_claims,
                            "sourceLinks": page.source_links,
                            "raw": page.raw,
                            "fallback": page.fallback,
                            "discoveredLinks": discovered_links,
                        }),
                        generated_at: current_timestamp(),
                    });
                }
            }
            "Web content fetched and normalized".to_string()
        }
        TaskKind::Enrich => {
            let records = store.records_for_objective(objective).await.unwrap_or_default();
            let extracted_pages = records.iter().filter(|record| record.provider == "firecrawl").count();
            let extracted_links = gather_links(&task.input, store, objective).await;
            evidence.push(EvidenceRecord {
                id: format!("ev-{}", task_id),
                objective: objective.to_string(),
                task_id: task_id.clone(),
                claim: format!("Supporting metadata attached to {} extracted page(s)", extracted_pages),
                source: provider_name.clone(),
                provider: provider_name.clone(),
                confidence: 0.72,
                contradictions: Vec::new(),
                notes: vec![format!("{} candidate link(s) available for enrichment", extracted_links.len())],
                payload: json!({
                    "task": task.kind_name(),
                    "objective": objective,
                    "summary": "Supporting metadata attached to the source set",
                    "evidenceCount": extracted_pages,
                    "candidateLinks": extracted_links,
                }),
                generated_at: current_timestamp(),
            });
            "Supporting metadata attached to the source set".to_string()
        }
        TaskKind::Verify => {
            let records = store.records_for_objective(objective).await.unwrap_or_default();
            let mesh = VerificationMesh::evaluate(objective.to_string(), &records);
            notes.push("Verification mesh cross-referenced initial claims against extracted content".to_string());
            let claim = mesh.summary.clone();
            evidence.push(EvidenceRecord {
                id: format!("ev-{}", task_id),
                objective: objective.to_string(),
                task_id: task_id.clone(),
                claim: claim.clone(),
                source: "verification-mesh".to_string(),
                provider: provider_name.clone(),
                confidence: mesh.confidence_score,
                contradictions: mesh.contradiction_notes.clone(),
                notes: mesh.hallucination_candidates.clone(),
                payload: json!({
                    "task": task.kind_name(),
                    "objective": objective,
                    "summary": mesh.summary,
                    "verification": mesh,
                }),
                generated_at: current_timestamp(),
            });
            "Claims checked against available evidence".to_string()
        }
        TaskKind::Synthesize => {
            let verification = latest_verification_report(store, objective).await;
            let summary = match verification.as_ref() {
                Some(report) => format!(
                    "Synthesis prepared from {} verified claim(s) and {} hallucination candidate(s)",
                    report.findings.iter().filter(|finding| matches!(finding.status, crate::verification::VerificationStatus::Verified)).count(),
                    report.hallucination_candidates.len()
                ),
                None => format!("Synthesis prepared from all completed waves for {}", objective),
            };
            evidence.push(EvidenceRecord {
                id: format!("ev-{}", task_id),
                objective: objective.to_string(),
                task_id: task_id.clone(),
                claim: summary.clone(),
                source: provider_name.clone(),
                provider: provider_name.clone(),
                confidence: 0.82,
                contradictions: Vec::new(),
                notes: vec!["Synthesis references verification output".to_string()],
                payload: json!({
                    "task": task.kind_name(),
                    "objective": objective,
                    "summary": summary.clone(),
                    "verification": verification,
                }),
                generated_at: current_timestamp(),
            });
            "Synthesis prepared from all completed waves".to_string()
        }
        TaskKind::Report => {
            let verification = latest_verification_report(store, objective).await;
            let summary = match verification.as_ref() {
                Some(report) => format!(
                    "Analyst-ready report assembled with {} finding(s) and {} contradiction note(s)",
                    report.findings.len(),
                    report.contradiction_notes.len()
                ),
                None => format!("Analyst-ready report payload prepared for {}", objective),
            };
            evidence.push(EvidenceRecord {
                id: format!("ev-{}", task_id),
                objective: objective.to_string(),
                task_id: task_id.clone(),
                claim: summary.clone(),
                source: provider_name.clone(),
                provider: provider_name.clone(),
                confidence: 0.9,
                contradictions: Vec::new(),
                notes: vec!["Final reporting payload assembled".to_string()],
                payload: json!({
                    "task": task.kind_name(),
                    "objective": objective,
                    "summary": summary.clone(),
                    "verification": verification,
                }),
                generated_at: current_timestamp(),
            });
            "Analyst-ready report payload prepared".to_string()
        }
        TaskKind::Custom(label) => {
            evidence.push(EvidenceRecord {
                id: format!("ev-{}", task_id),
                objective: objective.to_string(),
                task_id: task_id.clone(),
                claim: format!("Custom task '{}' completed", label),
                source: provider_name.clone(),
                provider: provider_name.clone(),
                confidence: 0.5,
                contradictions: Vec::new(),
                notes: skill.as_ref().map(|spec| vec![format!("Skill: {}", spec.name.clone())]).unwrap_or_default(),
                payload: json!({
                    "task": task.kind_name(),
                    "objective": objective,
                    "summary": format!("Custom task '{}' completed", label),
                }),
                generated_at: current_timestamp(),
            });
            format!("Custom task '{}' executed", label)
        }
    };

    if matches!(&kind, TaskKind::Plan) {
        evidence.push(EvidenceRecord {
            id: format!("ev-{}", task_id),
            objective: objective.to_string(),
            task_id: task_id.clone(),
            claim: format!("{} split into orchestrated tasks", objective),
            source: provider_name.clone(),
            provider: provider_name.clone(),
            confidence: 0.6,
            contradictions: Vec::new(),
            notes: skill.as_ref().map(|spec| vec![format!("Skill: {}", spec.name.clone())]).unwrap_or_default(),
            payload: json!({
                "task": task.kind_name(),
                "objective": objective,
                "summary": summary,
                "request": request,
            }),
            generated_at: current_timestamp(),
        });
    }

    TaskExecutionOutcome {
        task_id,
        kind,
        provider,
        success: true,
        summary,
        payload: json!({
            "task": task.kind_name(),
            "request": request,
        }),
        notes,
        evidence,
    }
}

async fn gather_links(task_input: &Value, store: &EvidenceGraph, objective: &str) -> Vec<String> {
    let mut links = BTreeSet::new();
    collect_links_from_value(task_input, &mut links);

    if links.is_empty() {
        if let Ok(records) = store.records_for_objective(objective).await {
            for record in records {
                collect_links_from_value(&record.payload, &mut links);
                if record.source.starts_with("http://") || record.source.starts_with("https://") {
                    links.insert(record.source);
                }
            }
        }
    }

    links.into_iter().take(12).collect()
}

fn collect_links_from_value(value: &Value, links: &mut BTreeSet<String>) {
    match value {
        Value::String(text) => {
            for token in extract_urls(text) {
                links.insert(token);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_links_from_value(item, links);
            }
        }
        Value::Object(map) => {
            for key in ["url", "href", "link", "sourceUrl", "sourceURL", "discoveredLink"] {
                if let Some(Value::String(text)) = map.get(key) {
                    for token in extract_urls(text) {
                        links.insert(token);
                    }
                }
            }
            for nested in map.values() {
                collect_links_from_value(nested, links);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn extract_urls(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let trimmed = token.trim_matches(|ch: char| matches!(ch, ',' | '.' | ')' | '(' | ']' | '[' | '"' | '\'' | ';'));
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                Some(trimmed.to_string())
            } else {
                None
            }
        })
        .collect()
}

async fn latest_verification_report(store: &EvidenceGraph, objective: &str) -> Option<VerificationMeshReport> {
    let records = store.records_for_objective(objective).await.ok()?;
    records
        .iter()
        .rev()
        .find_map(|record| {
            record
                .payload
                .get("verification")
                .cloned()
                .and_then(|value| serde_json::from_value::<VerificationMeshReport>(value).ok())
        })
}

fn skill_operation(skill: &SkillSpec) -> String {
    match &skill.capability {
        ProviderCapability::Plan => "chat/completions".to_string(),
        ProviderCapability::Reason => "chat/completions".to_string(),
        ProviderCapability::Search => "search".to_string(),
        ProviderCapability::Crawl => "crawl".to_string(),
        ProviderCapability::Extract => "extract".to_string(),
        ProviderCapability::Enrich => "enrich".to_string(),
        ProviderCapability::Report => "report".to_string(),
    }
}

impl Task {
    fn kind_name(&self) -> String {
        match &self.kind {
            TaskKind::Plan => "plan".to_string(),
            TaskKind::Search => "search".to_string(),
            TaskKind::Crawl => "crawl".to_string(),
            TaskKind::Extract => "extract".to_string(),
            TaskKind::Enrich => "enrich".to_string(),
            TaskKind::Verify => "verify".to_string(),
            TaskKind::Synthesize => "synthesize".to_string(),
            TaskKind::Report => "report".to_string(),
            TaskKind::Custom(label) => label.clone(),
        }
    }
}

fn current_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn slugify(input: &str) -> String {
    input
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
