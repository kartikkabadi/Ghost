use crate::db::EvidenceRecord;
use crate::orchestrator::{OrchestratorContract, TaskExecutionOutcome};
use crate::verification::{VerificationFinding, VerificationMeshReport, VerificationStatus};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchClaim {
    pub id: String,
    pub claim: String,
    pub confidence: f64,
    pub sources: Vec<String>,
    pub contradictions: Vec<String>,
    pub verification_status: Option<VerificationStatus>,
    pub evidence_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchBlindSpot {
    pub id: String,
    pub gap: String,
    pub impact: String,
    pub unmet_constraints: Vec<String>,
    pub suggested_follow_up_queries: Vec<String>,
    pub related_claims: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchReport {
    pub objective: String,
    pub summary: String,
    pub claims: Vec<ResearchClaim>,
    pub blind_spots: Vec<ResearchBlindSpot>,
    pub gaps: Vec<String>,
    pub next_queries: Vec<String>,
    pub confidence_score: f64,
    pub generated_at: String,
    pub markdown_brief: String,
    pub json_brief: Value,
}

#[derive(Clone, Debug)]
struct ClaimBundle {
    claim: String,
    verification_status: Option<VerificationStatus>,
    base_confidence: f64,
    evidence: Vec<EvidenceRecord>,
    contradictions: BTreeSet<String>,
}

#[derive(Clone, Default)]
pub struct ReportingEngine;

impl ReportingEngine {
    pub fn compile(
        &self,
        objective: String,
        contract: &OrchestratorContract,
        outcomes: Vec<TaskExecutionOutcome>,
        evidence: Vec<EvidenceRecord>,
    ) -> ResearchReport {
        let verification = latest_verification_report(&evidence);
        let bundles = match verification.as_ref() {
            Some(report) if !report.findings.is_empty() => self.bundles_from_verification(report, &evidence),
            _ => self.bundles_from_evidence(&evidence),
        };

        let claims = bundles
            .iter()
            .enumerate()
            .map(|(index, bundle)| {
                let source_set = bundle
                    .evidence
                    .iter()
                    .map(|record| format!("{}:{}", record.provider, record.source))
                    .collect::<BTreeSet<_>>();
                let mut sources: Vec<String> = source_set.into_iter().collect();
                if sources.is_empty() {
                    sources = fallback_sources_from_verification(&bundle.claim, verification.as_ref());
                }

                let confidence = claim_confidence(bundle, verification.as_ref());
                ResearchClaim {
                    id: format!("claim-{}", index + 1),
                    claim: bundle.claim.clone(),
                    confidence,
                    sources,
                    contradictions: bundle.contradictions.iter().cloned().collect(),
                    verification_status: bundle.verification_status.clone(),
                    evidence_count: bundle.evidence.len(),
                }
            })
            .collect::<Vec<_>>();

        let blind_spots = self.build_blind_spots(&claims, contract, &outcomes, verification.as_ref());
        let mut next_queries = merge_queries(&contract.next_queries, &blind_spots);
        if next_queries.is_empty() {
            next_queries.push(format!("Verify the objective '{}'", objective));
        }

        let gaps = merge_gaps(&contract.gaps, verification.as_ref(), &claims, &blind_spots, &outcomes);
        let confidence_score = if claims.is_empty() {
            0.0
        } else {
            claims.iter().map(|claim| claim.confidence).sum::<f64>() / claims.len() as f64
        };
        let summary = build_summary(&claims, &blind_spots, outcomes.len(), verification.as_ref());
        let generated_at = timestamp();
        let json_brief = json!({
            "objective": objective.clone(),
            "summary": summary.clone(),
            "claims": claims.clone(),
            "blindSpots": blind_spots.clone(),
            "gaps": gaps.clone(),
            "nextQueries": next_queries.clone(),
            "confidenceScore": confidence_score,
            "generatedAt": generated_at.clone(),
        });
        let markdown_brief = render_markdown(
            &objective,
            &summary,
            &claims,
            &blind_spots,
            &gaps,
            &next_queries,
            confidence_score,
            &generated_at,
        );

        ResearchReport {
            objective,
            summary,
            claims,
            blind_spots,
            gaps,
            next_queries,
            confidence_score,
            generated_at,
            markdown_brief,
            json_brief,
        }
    }

    fn bundles_from_verification(&self, report: &VerificationMeshReport, evidence: &[EvidenceRecord]) -> Vec<ClaimBundle> {
        report
            .findings
            .iter()
            .map(|finding| self.bundle_from_finding(finding, evidence))
            .collect()
    }

    fn bundles_from_evidence(&self, evidence: &[EvidenceRecord]) -> Vec<ClaimBundle> {
        let mut grouped: BTreeMap<String, Vec<EvidenceRecord>> = BTreeMap::new();
        for record in evidence {
            grouped.entry(record.claim.clone()).or_default().push(record.clone());
        }

        grouped
            .into_iter()
            .map(|(claim, records)| ClaimBundle {
                claim,
                verification_status: None,
                base_confidence: average_confidence(&records),
                evidence: records,
                contradictions: BTreeSet::new(),
            })
            .collect()
    }

    fn bundle_from_finding(&self, finding: &VerificationFinding, evidence: &[EvidenceRecord]) -> ClaimBundle {
        let mut related = evidence
            .iter()
            .filter(|record| record_matches_claim(record, &finding.claim))
            .cloned()
            .collect::<Vec<_>>();

        if related.is_empty() {
            related = evidence
                .iter()
                .filter(|record| record.task_id == "verify-1" || record.claim == finding.claim)
                .cloned()
                .collect::<Vec<_>>();
        }

        let mut contradictions = finding.contradictions.iter().cloned().collect::<BTreeSet<_>>();
        for record in &related {
            contradictions.extend(record.contradictions.iter().cloned());
        }

        ClaimBundle {
            claim: finding.claim.clone(),
            verification_status: Some(finding.status.clone()),
            base_confidence: match finding.status {
                VerificationStatus::Verified => 0.9,
                VerificationStatus::Conflicted => 0.55,
                VerificationStatus::Unverified => 0.2,
            },
            evidence: related,
            contradictions,
        }
    }

    fn build_blind_spots(
        &self,
        claims: &[ResearchClaim],
        contract: &OrchestratorContract,
        outcomes: &[TaskExecutionOutcome],
        verification: Option<&VerificationMeshReport>,
    ) -> Vec<ResearchBlindSpot> {
        let mut spots: BTreeMap<String, ResearchBlindSpot> = BTreeMap::new();

        for gap in &contract.gaps {
            insert_blind_spot(&mut spots, gap.clone(), "Constraint gap inherited from the orchestration contract".to_string(), vec![gap.clone()], contract.next_queries.clone(), Vec::new());
        }

        if verification.is_none() {
            insert_blind_spot(
                &mut spots,
                "No verification mesh output was available".to_string(),
                "Claims could not be promoted from raw evidence into a verified brief".to_string(),
                vec!["verification output missing".to_string()],
                vec![format!("Run the verifier stage for '{}'", contract.objective)],
                Vec::new(),
            );
        }

        for claim in claims {
            if claim.sources.is_empty() {
                insert_blind_spot(
                    &mut spots,
                    format!("Claim '{}' has no supporting sources", claim.claim),
                    "The report cannot show provenance for this claim".to_string(),
                    vec!["no source trail".to_string()],
                    vec![format!("Find at least one source that supports '{}'", claim.claim)],
                    vec![claim.id.clone()],
                );
            } else if claim.sources.len() == 1 {
                insert_blind_spot(
                    &mut spots,
                    format!("Claim '{}' is backed by a single source", claim.claim),
                    "Single-source findings remain fragile and should be corroborated".to_string(),
                    vec!["single-source support".to_string()],
                    vec![format!("Find another independent source for '{}'", claim.claim)],
                    vec![claim.id.clone()],
                );
            }

            if claim.confidence < 0.5 {
                insert_blind_spot(
                    &mut spots,
                    format!("Claim '{}' remains hypothesis-level", claim.claim),
                    "Confidence stayed below the threshold needed for a strong conclusion".to_string(),
                    vec!["low confidence".to_string()],
                    vec![format!("Recheck '{}' against independent evidence", claim.claim)],
                    vec![claim.id.clone()],
                );
            }

            if !claim.contradictions.is_empty() {
                insert_blind_spot(
                    &mut spots,
                    format!("Claim '{}' still has unresolved contradictions", claim.claim),
                    "Contradictory evidence needs adjudication before the claim is finalized".to_string(),
                    claim.contradictions.clone(),
                    vec![format!("Resolve contradictions around '{}'", claim.claim)],
                    vec![claim.id.clone()],
                );
            }
        }

        for outcome in outcomes {
            if !outcome.success {
                insert_blind_spot(
                    &mut spots,
                    format!("Task '{}' did not complete successfully", outcome.task_id),
                    "The execution graph has a coverage hole where evidence was expected".to_string(),
                    outcome.notes.clone(),
                    vec![format!("Rerun task '{}' with a narrower scope", outcome.task_id)],
                    Vec::new(),
                );
            }
        }

        if claims.is_empty() {
            insert_blind_spot(
                &mut spots,
                "No claims were promoted into the report".to_string(),
                "The run produced evidence, but not enough validated structure to answer the objective".to_string(),
                vec!["no promotable claims".to_string()],
                contract.next_queries.clone(),
                Vec::new(),
            );
        }

        spots.into_values().collect()
    }
}

fn average_confidence(records: &[EvidenceRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    records.iter().map(|record| record.confidence).sum::<f64>() / records.len() as f64
}

fn claim_confidence(bundle: &ClaimBundle, verification: Option<&VerificationMeshReport>) -> f64 {
    let evidence_score = average_confidence(&bundle.evidence);
    let status_score = bundle
        .verification_status
        .as_ref()
        .map(|status| match status {
            VerificationStatus::Verified => 0.95,
            VerificationStatus::Conflicted => 0.55,
            VerificationStatus::Unverified => 0.2,
        })
        .unwrap_or(bundle.base_confidence);
    let mut confidence = (evidence_score + status_score) / 2.0;

    if bundle.evidence.len() > 1 {
        confidence += 0.05;
    }
    if bundle.evidence.len() >= 3 {
        confidence += 0.03;
    }
    if !bundle.contradictions.is_empty() {
        confidence -= 0.15;
    }
    if let Some(report) = verification {
        if report.hallucination_candidates.iter().any(|candidate| candidate.eq_ignore_ascii_case(&bundle.claim)) {
            confidence -= 0.2;
        }
    }

    confidence.clamp(0.0, 1.0)
}

fn fallback_sources_from_verification(claim: &str, verification: Option<&VerificationMeshReport>) -> Vec<String> {
    verification
        .and_then(|report| {
            report.findings.iter().find(|finding| finding.claim == claim).map(|finding| {
                if finding.supporting_sources.is_empty() {
                    vec!["verification-mesh".to_string()]
                } else {
                    finding.supporting_sources.clone()
                }
            })
        })
        .unwrap_or_default()
}

fn merge_queries(contract_queries: &[String], blind_spots: &[ResearchBlindSpot]) -> Vec<String> {
    let mut queries = contract_queries.iter().cloned().collect::<BTreeSet<_>>();
    for spot in blind_spots {
        for query in &spot.suggested_follow_up_queries {
            queries.insert(query.clone());
        }
    }
    queries.into_iter().collect()
}

fn merge_gaps(
    contract_gaps: &[String],
    verification: Option<&VerificationMeshReport>,
    claims: &[ResearchClaim],
    blind_spots: &[ResearchBlindSpot],
    outcomes: &[TaskExecutionOutcome],
) -> Vec<String> {
    let mut gaps = contract_gaps.iter().cloned().collect::<BTreeSet<_>>();
    if let Some(report) = verification {
        if report.findings.iter().any(|finding| finding.status == VerificationStatus::Unverified) {
            gaps.insert("Some claims remain unverified after the verifier stage".to_string());
        }
        if !report.contradiction_notes.is_empty() {
            gaps.insert("Contradictions were observed but not fully resolved".to_string());
        }
        if !report.hallucination_candidates.is_empty() {
            gaps.insert("Potential hallucination candidates were identified".to_string());
        }
    } else {
        gaps.insert("No verifier output was available for structured synthesis".to_string());
    }

    if claims.iter().all(|claim| claim.sources.len() <= 1) && !claims.is_empty() {
        gaps.insert("The evidence set lacks source diversity".to_string());
    }

    if outcomes.iter().any(|outcome| !outcome.success) {
        gaps.insert("At least one pipeline task failed or returned incomplete output".to_string());
    }

    for spot in blind_spots {
        gaps.insert(spot.gap.clone());
    }

    gaps.into_iter().collect()
}

fn build_summary(
    claims: &[ResearchClaim],
    blind_spots: &[ResearchBlindSpot],
    outcome_count: usize,
    verification: Option<&VerificationMeshReport>,
) -> String {
    if claims.is_empty() {
        return format!(
            "The engine completed {} task outcome(s) but could not promote any claims into a verified brief.",
            outcome_count
        );
    }

    let verified_count = claims
        .iter()
        .filter(|claim| matches!(claim.verification_status.as_ref(), Some(VerificationStatus::Verified)))
        .count();
    let conflicted_count = claims
        .iter()
        .filter(|claim| matches!(claim.verification_status.as_ref(), Some(VerificationStatus::Conflicted)))
        .count();
    let unverified_count = claims
        .iter()
        .filter(|claim| matches!(claim.verification_status.as_ref(), Some(VerificationStatus::Unverified)))
        .count();

    let extra = verification
        .map(|report| format!(" The verifier surfaced {} contradiction note(s).", report.contradiction_notes.len()))
        .unwrap_or_default();

    format!(
        "The engine synthesized {} claim(s): {} verified, {} conflicted, {} unverified. {} blind spot(s) remain open.{}",
        claims.len(),
        verified_count,
        conflicted_count,
        unverified_count,
        blind_spots.len(),
        extra
    )
}

fn render_markdown(
    objective: &str,
    summary: &str,
    claims: &[ResearchClaim],
    blind_spots: &[ResearchBlindSpot],
    gaps: &[String],
    next_queries: &[String],
    confidence_score: f64,
    generated_at: &str,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Research brief\n\nObjective: {}\n\n", objective));
    out.push_str(&format!("Summary: {}\n\n", summary));
    out.push_str(&format!("Confidence score: {:.2}\n\n", confidence_score));
    out.push_str(&format!("Generated at: {}\n\n", generated_at));

    out.push_str("## Claims\n");
    if claims.is_empty() {
        out.push_str("- No claims were promoted.\n");
    } else {
        for claim in claims {
            let status = claim
                .verification_status
                .as_ref()
                .map(|status| format!("{:?}", status))
                .unwrap_or_else(|| "unclassified".to_string());
            out.push_str(&format!(
                "- {}\n  - status: {}\n  - confidence: {:.2}\n  - evidence count: {}\n  - sources: {}\n",
                claim.claim,
                status,
                claim.confidence,
                claim.evidence_count,
                if claim.sources.is_empty() { "none".to_string() } else { claim.sources.join(", ") }
            ));
            if !claim.contradictions.is_empty() {
                out.push_str(&format!("  - contradictions: {}\n", claim.contradictions.join(", ")));
            }
        }
    }

    out.push_str("\n## Blind spots\n");
    if blind_spots.is_empty() {
        out.push_str("- None identified.\n");
    } else {
        for spot in blind_spots {
            out.push_str(&format!(
                "- {}\n  - impact: {}\n  - unmet constraints: {}\n  - follow-up queries: {}\n",
                spot.gap,
                spot.impact,
                if spot.unmet_constraints.is_empty() { "none".to_string() } else { spot.unmet_constraints.join(", ") },
                if spot.suggested_follow_up_queries.is_empty() { "none".to_string() } else { spot.suggested_follow_up_queries.join("; ") }
            ));
        }
    }

    out.push_str("\n## Gaps\n");
    if gaps.is_empty() {
        out.push_str("- None\n");
    } else {
        for gap in gaps {
            out.push_str(&format!("- {}\n", gap));
        }
    }

    out.push_str("\n## Suggested follow-up queries\n");
    if next_queries.is_empty() {
        out.push_str("- None\n");
    } else {
        for query in next_queries {
            out.push_str(&format!("- {}\n", query));
        }
    }

    out
}

fn insert_blind_spot(
    spots: &mut BTreeMap<String, ResearchBlindSpot>,
    gap: String,
    impact: String,
    unmet_constraints: Vec<String>,
    follow_up_queries: Vec<String>,
    related_claims: Vec<String>,
) {
    spots
        .entry(gap.clone())
        .and_modify(|existing| {
            existing.unmet_constraints.extend(unmet_constraints.iter().cloned());
            existing.suggested_follow_up_queries.extend(follow_up_queries.iter().cloned());
            existing.related_claims.extend(related_claims.iter().cloned());
        })
        .or_insert_with(|| ResearchBlindSpot {
            id: format!("blind-{}", slugify(&gap)),
            gap,
            impact,
            unmet_constraints,
            suggested_follow_up_queries: follow_up_queries,
            related_claims,
        });
}

fn record_matches_claim(record: &EvidenceRecord, claim: &str) -> bool {
    if record.claim.eq_ignore_ascii_case(claim) {
        return true;
    }

    let claim_terms = meaningful_terms(claim);
    let evidence_text = record_text(record);
    let evidence_terms = meaningful_terms(&evidence_text);
    overlap_count(&claim_terms, &evidence_terms) > 0 || evidence_text.to_lowercase().contains(&claim.to_lowercase())
}

fn record_text(record: &EvidenceRecord) -> String {
    let mut fragments = vec![record.claim.clone(), record.source.clone(), record.provider.clone()];
    if let Some(summary) = record.payload.get("summary").and_then(Value::as_str) {
        fragments.push(summary.to_string());
    }
    if let Some(markdown) = record.payload.get("markdown").and_then(Value::as_str) {
        fragments.push(markdown.to_string());
    }
    if let Some(verification) = record.payload.get("verification") {
        fragments.push(verification.to_string());
    }
    fragments.join(" ")
}

fn meaningful_terms(text: &str) -> Vec<String> {
    let stopwords = [
        "the", "and", "with", "that", "this", "from", "into", "about", "your", "for", "are", "was", "were", "been", "will", "can", "could", "should", "would", "have", "has", "had", "not", "but", "when", "then", "what", "which", "who", "whom", "how", "why", "where", "there", "their", "they", "them", "you", "our", "out", "use", "used", "using", "over", "under", "more", "most", "some", "any", "all", "each", "per", "may", "might", "must", "shall",
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

fn latest_verification_report(evidence: &[EvidenceRecord]) -> Option<VerificationMeshReport> {
    evidence
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


fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verification::{VerificationFinding, VerificationMeshReport, VerificationStatus};

    fn sample_contract() -> OrchestratorContract {
        OrchestratorContract {
            objective: "find the reporting gaps".to_string(),
            tasks: Vec::new(),
            gaps: vec!["source coverage is partial".to_string()],
            next_queries: vec!["Check one more independent source".to_string()],
            max_waves: 4,
        }
    }

    fn sample_evidence() -> Vec<EvidenceRecord> {
        vec![
            EvidenceRecord {
                id: "e1".to_string(),
                objective: "find the reporting gaps".to_string(),
                task_id: "verify-1".to_string(),
                claim: "The system found one verified claim".to_string(),
                source: "verification-mesh".to_string(),
                provider: "xai".to_string(),
                confidence: 0.9,
                contradictions: vec!["missing citation".to_string()],
                notes: vec![],
                payload: json!({
                    "task": "verify",
                    "summary": "verification payload",
                    "verification": VerificationMeshReport {
                        objective: "find the reporting gaps".to_string(),
                        summary: "verification summary".to_string(),
                        findings: vec![
                            VerificationFinding {
                                claim: "The system found one verified claim".to_string(),
                                status: VerificationStatus::Verified,
                                supporting_sources: vec!["firecrawl:https://example.com".to_string()],
                                contradictions: vec![],
                                hallucination_risk: false,
                                rationale: "supported".to_string(),
                            },
                            VerificationFinding {
                                claim: "The system still needs more evidence".to_string(),
                                status: VerificationStatus::Unverified,
                                supporting_sources: vec![],
                                contradictions: vec!["timeout".to_string()],
                                hallucination_risk: true,
                                rationale: "not enough support".to_string(),
                            },
                        ],
                        contradiction_notes: vec!["timeout blocked cross-check".to_string()],
                        hallucination_candidates: vec!["The system still needs more evidence".to_string()],
                        confidence_score: 0.55,
                        generated_at: "1".to_string(),
                    }
                }),
                generated_at: "1".to_string(),
            },
            EvidenceRecord {
                id: "e2".to_string(),
                objective: "find the reporting gaps".to_string(),
                task_id: "crawl-1".to_string(),
                claim: "The system found one verified claim".to_string(),
                source: "https://example.com".to_string(),
                provider: "firecrawl".to_string(),
                confidence: 0.8,
                contradictions: vec![],
                notes: vec![],
                payload: json!({
                    "task": "extract",
                    "summary": "supporting extraction"
                }),
                generated_at: "2".to_string(),
            },
        ]
    }

    #[test]
    fn compile_creates_markdown_and_json_brief_with_blind_spots() {
        let engine = ReportingEngine::default();
        let report = engine.compile("find the reporting gaps".to_string(), &sample_contract(), Vec::new(), sample_evidence());

        assert!(!report.markdown_brief.is_empty());
        assert!(report.markdown_brief.contains("# Research brief"));
        assert!(report.json_brief.get("blindSpots").is_some());
        assert!(!report.blind_spots.is_empty());
        assert!(report.gaps.iter().any(|gap| gap.contains("source coverage")));
        assert!(report.next_queries.iter().any(|query| query.contains("independent source")));
    }
}
