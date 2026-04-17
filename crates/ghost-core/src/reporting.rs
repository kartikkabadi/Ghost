use crate::db::EvidenceRecord;
use crate::orchestrator::{OrchestrationPlan, TaskExecutionOutcome};
use crate::skills::SkillRegistry;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchClaim {
    pub id: String,
    pub claim: String,
    pub confidence: f32,
    pub sources: Vec<String>,
    pub contradictions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchReport {
    pub objective: String,
    pub summary: String,
    pub claims: Vec<ResearchClaim>,
    pub gaps: Vec<String>,
    pub next_queries: Vec<String>,
    pub confidence_score: f32,
    pub generated_at: String,
}

#[derive(Clone, Default)]
pub struct ReportingEngine;

impl ReportingEngine {
    pub fn compile(
        &self,
        objective: String,
        plan: OrchestrationPlan,
        outcomes: Vec<TaskExecutionOutcome>,
        evidence: Vec<EvidenceRecord>,
    ) -> ResearchReport {
        let mut grouped: BTreeMap<String, Vec<EvidenceRecord>> = BTreeMap::new();
        for record in evidence {
            grouped.entry(record.claim.clone()).or_default().push(record);
        }

        let claims = grouped
            .into_iter()
            .enumerate()
            .map(|(index, (claim, records))| {
                let confidence = if records.is_empty() {
                    0.0
                } else {
                    records.iter().map(|record| record.confidence).sum::<f32>() / records.len() as f32
                };
                let mut sources = Vec::new();
                let mut contradictions = Vec::new();
                for record in &records {
                    sources.push(format!("{}:{}", record.provider, record.source));
                    contradictions.extend(record.contradictions.clone());
                }
                ResearchClaim {
                    id: format!("claim-{}", index + 1),
                    claim,
                    confidence,
                    sources,
                    contradictions,
                }
            })
            .collect::<Vec<_>>();

        let mut next_queries = plan.next_queries.clone();
        if next_queries.is_empty() {
            next_queries.push(format!("Verify the objective '{}'", objective));
        }

        let summary = if claims.is_empty() {
            format!(
                "The engine completed {} planned task(s) but did not promote any claims yet.",
                outcomes.len()
            )
        } else {
            format!(
                "The engine produced {} evidence-backed claim(s) from {} task outcome(s).",
                claims.len(),
                outcomes.len()
            )
        };

        let confidence_score = if claims.is_empty() {
            0.0
        } else {
            claims.iter().map(|claim| claim.confidence).sum::<f32>() / claims.len() as f32
        };

        ResearchReport {
            objective,
            summary,
            claims,
            gaps: plan.gaps,
            next_queries,
            confidence_score,
            generated_at: timestamp(),
        }
    }
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
