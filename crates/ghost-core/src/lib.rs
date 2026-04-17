pub mod db;
pub mod gateway;
pub mod orchestrator;
pub mod provider;
pub mod reporting;
pub mod skills;

pub use db::{EvidenceGraph, EvidenceRecord};
pub use gateway::{ByokProviderGateway, GatewayError, GatewayRequest, GatewayResponse};
pub use orchestrator::{OrchestrationPlan, OrchestratorContract, Task, TaskBroker, TaskExecutionOutcome, TaskKind, WaveScheduler};
pub use provider::{ProviderCapability, ProviderKind, ProviderRuntime, ProviderTarget};
pub use reporting::{ResearchClaim, ResearchReport, ReportingEngine};
pub use skills::{SkillRegistry, SkillSpec};

pub struct GhostCore {
    gateway: ByokProviderGateway,
    skills: SkillRegistry,
    store: EvidenceGraph,
    reporter: ReportingEngine,
}

impl GhostCore {
    pub fn new() -> Self {
        Self {
            gateway: ByokProviderGateway::new(),
            skills: SkillRegistry::default(),
            store: EvidenceGraph::default(),
            reporter: ReportingEngine::default(),
        }
    }

    pub fn with_runtime(gateway: ByokProviderGateway, skills: SkillRegistry) -> Self {
        Self {
            gateway,
            skills,
            store: EvidenceGraph::default(),
            reporter: ReportingEngine::default(),
        }
    }

    pub async fn run(&self, objective: impl Into<String>) -> Result<ResearchReport, orchestrator::OrchestratorError> {
        let objective = objective.into();
        let contract = OrchestratorContract::from_objective(objective.clone(), &self.skills);
        let broker = TaskBroker::new(contract.clone());
        let gateway = self.gateway.clone();
        let skills = self.skills.clone();
        let store = self.store.clone();

        let outcomes = broker
            .execute_with(move |task| {
                let gateway = gateway.clone();
                let skills = skills.clone();
                Box::pin(async move {
                    let output = orchestrator::simulate_task(&gateway, &skills, task.clone()).await;
                    output
                })
            })
            .await?;

        for outcome in &outcomes {
            for record in &outcome.evidence {
                store.insert(record.clone()).await;
            }
        }

        Ok(self.reporter.compile(objective, contract, outcomes, store.snapshot().await))
    }
}

impl Default for GhostCore {
    fn default() -> Self {
        Self::new()
    }
}
