use crate::db::EvidenceRecord;
use crate::gateway::{ByokProviderGateway, GatewayRequest};
use crate::provider::{ProviderCapability, ProviderKind};
use crate::skills::{SkillRegistry, SkillSpec};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
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
        let mut investigation_task_ids = Vec::new();

        let planner = Task {
            id: "plan-1".to_string(),
            kind: TaskKind::Plan,
            skill_id: registry
                .get("planner.openrouter")
                .map(|skill| skill.id.clone()),
            provider: Some(ProviderKind::OpenRouter),
            input: json!({"objective": objective.clone()}),
            depends_on: Vec::new(),
        };
        tasks.push(planner);

        for skill in registry.ordered_parallel_skills() {
            if matches!(skill.capability, ProviderCapability::Plan) {
                continue;
            }
            let task_id = format!("{}-task", skill.id.replace('.', "-"));
            let kind = match skill.capability {
                ProviderCapability::Search => TaskKind::Search,
                ProviderCapability::Crawl => TaskKind::Crawl,
                ProviderCapability::Extract => TaskKind::Extract,
                ProviderCapability::Enrich => TaskKind::Enrich,
                ProviderCapability::Reason | ProviderCapability::Report | ProviderCapability::Plan => TaskKind::Custom(skill.id.clone()),
            };
            investigation_task_ids.push(task_id.clone());
            tasks.push(Task {
                id: task_id,
                kind,
                skill_id: Some(skill.id.clone()),
                provider: Some(skill.provider.clone()),
                input: json!({
                    "objective": objective.clone(),
                    "skill": skill.id,
                    "capability": format!("{:?}", skill.capability),
                    "tags": skill.tags,
                }),
                depends_on: vec!["plan-1".to_string()],
            });
        }

        let synthesis = Task {
            id: "synthesis-1".to_string(),
            kind: TaskKind::Synthesize,
            skill_id: registry.get("reasoner.xai").map(|skill| skill.id.clone()),
            provider: Some(ProviderKind::Xai),
            input: json!({"objective": objective.clone()}),
            depends_on: investigation_task_ids.clone(),
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
pub struct OrchestratorContract {
    pub objective: String,
    pub plan: OrchestrationPlan,
    pub max_waves: usize,
}

impl OrchestratorContract {
    pub fn from_objective(objective: impl Into<String>, registry: &SkillRegistry) -> Self {
        let objective = objective.into();
        Self {
            plan: OrchestrationPlan::from_objective(objective.clone(), registry),
            objective,
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
        self.scheduler.execute(self.contract.plan.tasks.clone(), Arc::new(executor)).await
    }
}

pub async fn simulate_task(
    gateway: &ByokProviderGateway,
    skills: &SkillRegistry,
    task: Task,
) -> TaskExecutionOutcome {
    let kind = task.kind.clone();
    let provider = task.provider.clone();
    let skill = task
        .skill_id
        .as_deref()
        .and_then(|skill_id| skills.get(skill_id))
        .cloned();

    let request: Option<GatewayRequest> = provider
        .clone()
        .map(|provider_kind| {
            let operation = skill
                .as_ref()
                .map(|skill_spec| skill_operation(skill_spec))
                .unwrap_or_else(|| task.kind_name());
            gateway.plan_request(provider_kind, operation, task.input.clone()).ok()
        })
        .flatten();

    let summary = match &kind {
        TaskKind::Plan => "Objective decomposed into a wave-based plan".to_string(),
        TaskKind::Search => "Search candidates collected".to_string(),
        TaskKind::Crawl => "Web content fetched and normalized".to_string(),
        TaskKind::Extract => "Structured fields extracted from source material".to_string(),
        TaskKind::Enrich => "Search context enriched with supporting metadata".to_string(),
        TaskKind::Verify => "Claims checked against available evidence".to_string(),
        TaskKind::Synthesize => "Synthesis prepared from all completed waves".to_string(),
        TaskKind::Report => "Analyst-ready report payload prepared".to_string(),
        TaskKind::Custom(label) => format!("Custom task '{}' executed", label),
    };

    let claim = match &kind {
        TaskKind::Plan => format!("{} split into orchestrated tasks", task.input.get("objective").and_then(Value::as_str).unwrap_or("objective")),
        TaskKind::Search => "Candidate source set identified".to_string(),
        TaskKind::Crawl => "Primary sources fetched for analysis".to_string(),
        TaskKind::Extract => "Structured evidence extracted from source payloads".to_string(),
        TaskKind::Enrich => "Supporting metadata attached to the source set".to_string(),
        TaskKind::Verify => "Key claims were checked for consistency".to_string(),
        TaskKind::Synthesize => "Validated findings were synthesized into a reportable frame".to_string(),
        TaskKind::Report => "Final report payload assembled for review".to_string(),
        TaskKind::Custom(label) => format!("Custom task '{}' completed", label),
    };

    let provider_name = provider.as_ref().map(|p| format!("{:?}", p)).unwrap_or_else(|| "local".to_string());
    let mut evidence = Vec::new();
    if matches!(&kind, TaskKind::Plan | TaskKind::Search | TaskKind::Crawl | TaskKind::Extract | TaskKind::Enrich | TaskKind::Verify | TaskKind::Synthesize | TaskKind::Report | TaskKind::Custom(_)) {
        evidence.push(EvidenceRecord {
            id: format!("ev-{}", task.id),
            objective: task
                .input
                .get("objective")
                .and_then(Value::as_str)
                .unwrap_or("unknown objective")
                .to_string(),
            task_id: task.id.clone(),
            claim,
            source: request
                .as_ref()
                .map(|req| req.url.clone())
                .unwrap_or_else(|| provider_name.clone()),
            provider: provider_name,
            confidence: match &kind {
                TaskKind::Plan => 0.6,
                TaskKind::Search | TaskKind::Crawl => 0.7,
                TaskKind::Extract | TaskKind::Enrich => 0.75,
                TaskKind::Verify => 0.85,
                TaskKind::Synthesize => 0.8,
                TaskKind::Report => 0.9,
                TaskKind::Custom(_) => 0.5,
            },
            contradictions: Vec::new(),
            notes: skill.map(|spec| vec![format!("Skill: {}", spec.name)]).unwrap_or_default(),
            payload: json!({
                "summary": summary,
                "request": request,
            }),
            generated_at: current_timestamp(),
        });
    }

    TaskExecutionOutcome {
        task_id: task.id,
        kind,
        provider,
        success: true,
        summary,
        payload: json!({
            "task": task.kind_name(),
            "request": request,
        }),
        notes: vec!["Executed by the wave scheduler".to_string()],
        evidence,
    }
}

fn skill_operation(skill: &SkillSpec) -> String {
    match skill.capability {
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
