use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: String,
    pub objective: String,
    pub task_id: String,
    pub claim: String,
    pub source: String,
    pub provider: String,
    pub confidence: f32,
    pub contradictions: Vec<String>,
    pub notes: Vec<String>,
    pub payload: Value,
    pub generated_at: String,
}

#[derive(Clone, Default)]
pub struct EvidenceGraph {
    inner: Arc<RwLock<HashMap<String, Vec<EvidenceRecord>>>>,
}

impl EvidenceGraph {
    pub async fn insert(&self, record: EvidenceRecord) {
        let mut guard = self.inner.write().await;
        guard.entry(record.objective.clone()).or_default().push(record);
    }

    pub async fn records_for_objective(&self, objective: &str) -> Vec<EvidenceRecord> {
        let guard = self.inner.read().await;
        guard.get(objective).cloned().unwrap_or_default()
    }

    pub async fn snapshot(&self) -> Vec<EvidenceRecord> {
        let guard = self.inner.read().await;
        guard.values().flat_map(|records| records.clone()).collect()
    }
}
