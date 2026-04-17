use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRecord {
    pub id: String,
    pub objective: String,
    pub task_id: String,
    pub claim: String,
    pub source: String,
    pub provider: String,
    pub confidence: f64,
    pub contradictions: Vec<String>,
    pub notes: Vec<String>,
    pub payload: Value,
    pub generated_at: String,
}

#[derive(Clone, Debug)]
pub struct EvidenceGraph {
    path: PathBuf,
}

impl EvidenceGraph {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn database_path(&self) -> &PathBuf {
        &self.path
    }

    pub async fn insert(&self, record: EvidenceRecord) -> SqlResult<()> {
        self.with_connection(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS evidence_records (
                    id TEXT PRIMARY KEY,
                    objective TEXT NOT NULL,
                    task_id TEXT NOT NULL,
                    claim TEXT NOT NULL,
                    source TEXT NOT NULL,
                    provider TEXT NOT NULL,
                    confidence REAL NOT NULL,
                    contradictions TEXT NOT NULL,
                    notes TEXT NOT NULL,
                    payload TEXT NOT NULL,
                    generated_at TEXT NOT NULL
                )",
                [],
            )?;
            conn.execute(
                "INSERT OR REPLACE INTO evidence_records (
                    id, objective, task_id, claim, source, provider, confidence,
                    contradictions, notes, payload, generated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    record.id,
                    record.objective,
                    record.task_id,
                    record.claim,
                    record.source,
                    record.provider,
                    record.confidence,
                    serde_json::to_string(&record.contradictions).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&record.notes).unwrap_or_else(|_| "[]".to_string()),
                    serde_json::to_string(&record.payload).unwrap_or_else(|_| "null".to_string()),
                    record.generated_at,
                ],
            )?;
            Ok(())
        })
    }

    pub async fn records_for_objective(&self, objective: &str) -> SqlResult<Vec<EvidenceRecord>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, objective, task_id, claim, source, provider, confidence, contradictions, notes, payload, generated_at
                 FROM evidence_records
                 WHERE objective = ?1
                 ORDER BY generated_at ASC, id ASC",
            )?;
            let rows = stmt.query_map(params![objective], |row| row_to_record(row))?;
            rows.collect()
        })
    }

    pub async fn snapshot(&self) -> SqlResult<Vec<EvidenceRecord>> {
        self.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, objective, task_id, claim, source, provider, confidence, contradictions, notes, payload, generated_at
                 FROM evidence_records
                 ORDER BY generated_at ASC, id ASC",
            )?;
            let rows = stmt.query_map([], |row| row_to_record(row))?;
            rows.collect()
        })
    }

    fn with_connection<T>(&self, f: impl FnOnce(&Connection) -> SqlResult<T>) -> SqlResult<T> {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let conn = Connection::open(&self.path)?;
        f(&conn)
    }
}

impl Default for EvidenceGraph {
    fn default() -> Self {
        let path = env::var_os("GHOST_EVIDENCE_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(".ghost")
                    .join("evidence.sqlite")
            });
        Self::new(path)
    }
}

fn row_to_record(row: &rusqlite::Row<'_>) -> SqlResult<EvidenceRecord> {
    let contradictions: String = row.get(7)?;
    let notes: String = row.get(8)?;
    let payload: String = row.get(9)?;

    Ok(EvidenceRecord {
        id: row.get(0)?,
        objective: row.get(1)?,
        task_id: row.get(2)?,
        claim: row.get(3)?,
        source: row.get(4)?,
        provider: row.get(5)?,
        confidence: row.get(6)?,
        contradictions: serde_json::from_str(&contradictions).unwrap_or_default(),
        notes: serde_json::from_str(&notes).unwrap_or_default(),
        payload: serde_json::from_str(&payload).unwrap_or(Value::Null),
        generated_at: row.get(10)?,
    })
}
