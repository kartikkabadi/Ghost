//! KAR-161: Recursive search primitives.
//!
//! Implements the `recursive-search` OSINT primitive defined in `config.yaml`:
//! expand seeds through pivot rules up to a configurable depth, deduplicating
//! candidate sources and running contradiction-scan markers at each level.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// ─── Pivot rule ───────────────────────────────────────────────────────────────

/// A single transformation rule that expands a seed query into one or more
/// derivative queries.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PivotRule {
    /// Human-readable label, e.g. `"domain pivot"` or `"username handle"`.
    pub label: String,
    /// Pattern prefix injected before the seed (e.g. `"site:"`), or `None` to
    /// use the seed as-is.
    pub prefix: Option<String>,
    /// Pattern suffix appended after the seed (e.g. `" filetype:pdf"`), or
    /// `None` for no suffix.
    pub suffix: Option<String>,
    /// If `true`, also emit the raw seed query unchanged at this level.
    pub include_raw: bool,
}

impl PivotRule {
    /// Expand a single seed using this rule. Returns one or two query strings:
    /// the transformed version and optionally the raw seed.
    pub fn expand(&self, seed: &str) -> Vec<String> {
        let transformed = format!(
            "{}{}{}",
            self.prefix.as_deref().unwrap_or(""),
            seed,
            self.suffix.as_deref().unwrap_or("")
        );
        let mut out = vec![transformed];
        if self.include_raw {
            out.push(seed.to_string());
        }
        out
    }
}

// ─── Result types ─────────────────────────────────────────────────────────────

/// Per-depth metadata recorded during the recursive expansion.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepthRecord {
    /// Zero-based depth level this record was produced at.
    pub depth: usize,
    /// Queries generated at this depth before deduplication.
    pub queries: Vec<String>,
    /// `true` if a contradiction-scan pass should be run on sources found at
    /// this depth (currently always `true` for depth > 0).
    pub contradiction_scan: bool,
}

/// Output of a full `RecursiveSearchEngine::run` call.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecursiveSearchResult {
    /// Final de-duplicated, depth-limited set of expanded queries.
    pub expanded_queries: Vec<String>,
    /// Unique candidate source URLs / identifiers surfaced by the expansion
    /// (populated when the caller provides a `source_resolver`).
    pub candidate_sources: Vec<String>,
    /// Per-depth metadata, ordered from shallowest to deepest.
    pub depth_records: Vec<DepthRecord>,
    /// Total number of duplicate queries pruned across all depths.
    pub pruned_duplicates: usize,
}

// ─── Engine ───────────────────────────────────────────────────────────────────

/// Configuration for a single recursive expansion run.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecursiveSearchConfig {
    /// Initial seed queries to start the expansion from.
    pub seed_queries: Vec<String>,
    /// Pivot rules applied at each depth level.
    pub pivot_rules: Vec<PivotRule>,
    /// Maximum expansion depth (inclusive). Depth 0 = seeds only.
    pub depth_limit: usize,
    /// Optional list of already-known source URLs; used for deduplication of
    /// `candidate_sources` only.
    #[serde(default)]
    pub known_sources: Vec<String>,
}

/// Engine that implements the KAR-161 recursive source-expansion loop.
pub struct RecursiveSearchEngine;

impl RecursiveSearchEngine {
    /// Run the recursive expansion described by `config`.
    ///
    /// `source_resolver` is an optional async function that, given a query
    /// string, returns candidate source URLs. Pass `None` to skip live
    /// resolution (useful in tests or when sources are resolved externally).
    ///
    /// The function is synchronous; callers that need async resolution should
    /// collect sources separately and pass them in via `known_sources`.
    pub fn run(config: &RecursiveSearchConfig) -> RecursiveSearchResult {
        let mut seen_queries: BTreeSet<String> = BTreeSet::new();
        let mut seen_sources: BTreeSet<String> =
            config.known_sources.iter().cloned().collect();
        let mut depth_records: Vec<DepthRecord> = Vec::new();
        let mut pruned_duplicates: usize = 0;

        // Seed queries are depth 0
        let mut current_wave: Vec<String> = config
            .seed_queries
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        for depth in 0..=config.depth_limit {
            // Deduplicate the current wave against everything seen so far
            let mut deduped: Vec<String> = Vec::new();
            for q in &current_wave {
                if seen_queries.insert(q.clone()) {
                    deduped.push(q.clone());
                } else {
                    pruned_duplicates += 1;
                }
            }

            depth_records.push(DepthRecord {
                depth,
                queries: deduped.clone(),
                contradiction_scan: depth > 0,
            });

            // Stop expanding at the final depth
            if depth == config.depth_limit {
                break;
            }

            // Expand each non-duplicate query through every pivot rule
            let mut next_wave: Vec<String> = Vec::new();
            for query in &deduped {
                for rule in &config.pivot_rules {
                    next_wave.extend(rule.expand(query));
                }
            }
            current_wave = next_wave;
        }

        let expanded_queries: Vec<String> = seen_queries.into_iter().collect();

        // `candidate_sources` is populated externally (e.g. by a search skill
        // that calls Exa/Firecrawl). Here we return the deduped known_sources
        // list so callers get a consistent type.
        let candidate_sources: Vec<String> = seen_sources.into_iter().collect();

        RecursiveSearchResult {
            expanded_queries,
            candidate_sources,
            depth_records,
            pruned_duplicates,
        }
    }

    /// Convenience method: merge externally-resolved source URLs into a
    /// `RecursiveSearchResult`, deduplicating against what was already present.
    pub fn merge_sources(
        result: &mut RecursiveSearchResult,
        new_sources: impl IntoIterator<Item = String>,
    ) {
        let existing: BTreeSet<String> = result.candidate_sources.iter().cloned().collect();
        let mut merged: BTreeSet<String> = existing;
        for s in new_sources {
            merged.insert(s);
        }
        result.candidate_sources = merged.into_iter().collect();
    }
}

// ─── Default pivot rules aligned with config.yaml osint_primitives ────────────

impl RecursiveSearchConfig {
    /// Build a `RecursiveSearchConfig` with the default OSINT pivot rules
    /// described in `config.yaml` (domain, filetype, social pivots).
    pub fn with_default_pivots(seeds: Vec<String>, depth_limit: usize) -> Self {
        Self {
            seed_queries: seeds,
            pivot_rules: default_osint_pivot_rules(),
            depth_limit,
            known_sources: Vec::new(),
        }
    }
}

/// Default pivot rules matching the `recursive-search` osint primitive from
/// `config.yaml`.
fn default_osint_pivot_rules() -> Vec<PivotRule> {
    vec![
        PivotRule {
            label: "domain pivot".to_string(),
            prefix: Some("site:".to_string()),
            suffix: None,
            include_raw: true,
        },
        PivotRule {
            label: "filetype pivot – pdf".to_string(),
            prefix: None,
            suffix: Some(" filetype:pdf".to_string()),
            include_raw: false,
        },
        PivotRule {
            label: "intitle pivot".to_string(),
            prefix: Some("intitle:".to_string()),
            suffix: None,
            include_raw: false,
        },
        PivotRule {
            label: "inurl pivot".to_string(),
            prefix: Some("inurl:".to_string()),
            suffix: None,
            include_raw: false,
        },
        PivotRule {
            label: "cache pivot".to_string(),
            prefix: Some("cache:".to_string()),
            suffix: None,
            include_raw: false,
        },
    ]
}
