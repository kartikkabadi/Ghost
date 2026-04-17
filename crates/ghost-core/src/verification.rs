use crate::db::EvidenceRecord;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Conflicted,
    Unverified,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationFinding {
    pub claim: String,
    pub status: VerificationStatus,
    pub supporting_sources: Vec<String>,
    pub contradictions: Vec<String>,
    pub hallucination_risk: bool,
    pub rationale: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMeshReport {
    pub objective: String,
    pub summary: String,
    pub findings: Vec<VerificationFinding>,
    pub contradiction_notes: Vec<String>,
    pub hallucination_candidates: Vec<String>,
    pub confidence_score: f64,
    pub generated_at: String,
}

#[derive(Clone, Default)]
pub struct VerificationMesh;

impl VerificationMesh {
    pub fn evaluate(objective: String, evidence: &[EvidenceRecord]) -> VerificationMeshReport {
        let initial_claims = evidence
            .iter()
            .filter(|record| matches_task(record, &["plan", "search", "reason", "reasoner", "custom"]))
            .collect::<Vec<_>>();
        let extracted_records = evidence
            .iter()
            .filter(|record| matches_task(record, &["crawl", "extract", "enrich"]))
            .collect::<Vec<_>>();

        let mut findings = Vec::new();
        let mut contradiction_notes = BTreeSet::new();
        let mut hallucination_candidates = BTreeSet::new();
        let mut confidence_scores = Vec::new();

        for claim_record in initial_claims {
            let claim_text = claim_record.claim.trim().to_string();
            let claim_terms = meaningful_terms(&claim_text);
            let mut supporting_sources = BTreeSet::new();
            let mut contradictions = BTreeSet::new();
            let mut support_hits = 0usize;

            for extracted in &extracted_records {
                let evidence_text = evidence_text(extracted);
                let evidence_terms = meaningful_terms(&evidence_text);
                let overlap = overlap_count(&claim_terms, &evidence_terms);
                let evidence_lower = evidence_text.to_lowercase();
                let claim_lower = claim_text.to_lowercase();

                if overlap > 0 || evidence_lower.contains(&claim_lower) {
                    support_hits += 1;
                    supporting_sources.insert(format!("{}:{}", extracted.provider, extracted.source));
                }

                if is_contradictory(&claim_text, &evidence_text, overlap) {
                    contradictions.insert(format!("{} contradicts {}", extracted.source, claim_record.id));
                    contradiction_notes.insert(format!("{} may contradict '{}'", extracted.source, claim_text));
                }
            }

            let status = if support_hits > 0 && contradictions.is_empty() {
                VerificationStatus::Verified
            } else if support_hits > 0 {
                VerificationStatus::Conflicted
            } else {
                VerificationStatus::Unverified
            };

            let hallucination_risk = support_hits == 0;
            if hallucination_risk {
                hallucination_candidates.insert(claim_text.clone());
            }

            let rationale = match status {
                VerificationStatus::Verified => format!("{} supporting source(s) cross-reference this claim", supporting_sources.len()),
                VerificationStatus::Conflicted => format!("{} supporting source(s) found, but contradictions remain", supporting_sources.len()),
                VerificationStatus::Unverified => "No extracted evidence matched the claim terms strongly enough to verify it".to_string(),
            };

            let confidence = match status {
                VerificationStatus::Verified => 0.9,
                VerificationStatus::Conflicted => 0.55,
                VerificationStatus::Unverified => 0.2,
            };
            confidence_scores.push(confidence);

            findings.push(VerificationFinding {
                claim: claim_text,
                status,
                supporting_sources: supporting_sources.into_iter().collect(),
                contradictions: contradictions.into_iter().collect(),
                hallucination_risk,
                rationale,
            });
        }

        let supported = findings.iter().filter(|finding| finding.status == VerificationStatus::Verified).count();
        let summary = if findings.is_empty() {
            format!("Verification mesh found no claims to evaluate for {}", objective)
        } else {
            format!(
                "Verification mesh checked {} claim(s): {} verified, {} conflicted, {} unverified.",
                findings.len(),
                supported,
                findings.iter().filter(|finding| finding.status == VerificationStatus::Conflicted).count(),
                findings.iter().filter(|finding| finding.status == VerificationStatus::Unverified).count()
            )
        };

        let confidence_score = if confidence_scores.is_empty() {
            0.0
        } else {
            confidence_scores.iter().sum::<f64>() / confidence_scores.len() as f64
        };

        VerificationMeshReport {
            objective,
            summary,
            findings,
            contradiction_notes: contradiction_notes.into_iter().collect(),
            hallucination_candidates: hallucination_candidates.into_iter().collect(),
            confidence_score,
            generated_at: timestamp(),
        }
    }
}

fn matches_task(record: &EvidenceRecord, accepted: &[&str]) -> bool {
    record
        .payload
        .get("task")
        .and_then(Value::as_str)
        .map(|task| accepted.iter().any(|candidate| task.eq_ignore_ascii_case(candidate)) || task.to_ascii_lowercase().contains("reasoner"))
        .unwrap_or(false)
}

fn evidence_text(record: &EvidenceRecord) -> String {
    let mut fragments = vec![record.claim.clone(), record.source.clone()];
    if let Some(summary) = record.payload.get("summary").and_then(Value::as_str) {
        fragments.push(summary.to_string());
    }
    if let Some(markdown) = record.payload.get("markdown").and_then(Value::as_str) {
        fragments.push(markdown.to_string());
    }
    if let Some(summary) = record.payload.get("firecrawl").and_then(Value::as_bool) {
        if summary {
            fragments.push("firecrawl".to_string());
        }
    }
    fragments.join(" ")
}

fn meaningful_terms(text: &str) -> Vec<String> {
    let stopwords = [
        "the", "and", "with", "that", "this", "from", "into", "about", "into", "your", "for", "are", "was", "were", "been", "will", "can", "could", "should", "would", "have", "has", "had", "not", "but", "when", "then", "what", "which", "who", "whom", "how", "why", "where", "there", "their", "they", "them", "you", "our", "out", "use", "used", "using", "over", "under", "more", "most", "some", "any", "all", "each", "per", "can", "may", "might", "must", "shall", "who", "what", "why", "how",
    ];
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter_map(|token| {
            let lowered = token.trim().to_lowercase();
            if lowered.len() < 4 || stopwords.contains(&lowered.as_str()) {
                None
            } else {
                Some(lowered)
            }
        })
        .collect()
}

fn overlap_count(left: &[String], right: &[String]) -> usize {
    let right_set: BTreeSet<&str> = right.iter().map(|term| term.as_str()).collect();
    left.iter().filter(|term| right_set.contains(term.as_str())).count()
}

fn is_contradictory(claim: &str, evidence: &str, overlap: usize) -> bool {
    if overlap == 0 {
        return false;
    }

    let claim_lower = claim.to_lowercase();
    let evidence_lower = evidence.to_lowercase();
    let negations = [" not ", " no ", " never ", " without ", " none ", "cannot", "can't", "fails", "failure", "contradict", "contradiction"];

    if negations.iter().any(|needle| evidence_lower.contains(needle)) {
        return true;
    }

    let claim_numbers = numbers(&claim_lower);
    let evidence_numbers = numbers(&evidence_lower);
    if !claim_numbers.is_empty() && !evidence_numbers.is_empty() && claim_numbers != evidence_numbers {
        return true;
    }

    false
}

fn numbers(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_ascii_digit())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_string())
        .collect()
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
