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
    // PR #10 blocker: missing serde defaults – empty vec on missing field
    #[serde(default)]
    pub contradictions: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    pub payload: Value,
    pub generated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMemoryRecord {
    pub session_id: String,
    pub objective: String,
    pub key: String,
    pub value: Value,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineEventRecord {
    pub session_id: String,
    pub objective: String,
    pub kind: String,
    pub detail: String,
    pub payload: Value,
    pub created_at: String,
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
            self.ensure_schema(conn)?;
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

    /// PR #10 blocker fix: non-atomic bulk deletes / inserts.
    /// Wraps N inserts inside an explicit transaction; the entire batch is
    /// rolled back if any single row fails – no partial writes.
    pub async fn bulk_insert_records(&self, records: Vec<EvidenceRecord>) -> SqlResult<()> {
        self.with_connection(|conn| {
            self.ensure_schema(conn)?;
            // Begin explicit transaction to make the batch atomic
            conn.execute_batch("BEGIN DEFERRED")?;
            let result = (|| -> SqlResult<()> {
                for record in &records {
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
                }
                Ok(())
            })();
            match result {
                Ok(()) => {
                    conn.execute_batch("COMMIT")?;
                    Ok(())
                }
                Err(e) => {
                    let _ = conn.execute_batch("ROLLBACK");
                    Err(e)
                }
            }
        })
    }

    pub async fn upsert_session_memory(&self, record: SessionMemoryRecord) -> SqlResult<()> {
        self.with_connection(|conn| {
            self.ensure_schema(conn)?;
            conn.execute(
                "INSERT OR REPLACE INTO session_memory (
                    session_id, objective, key, value, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    record.session_id,
                    record.objective,
                    record.key,
                    serde_json::to_string(&record.value).unwrap_or_else(|_| "null".to_string()),
                    record.created_at,
                ],
            )?;
            Ok(())
        })
    }

    pub async fn append_engine_event(&self, event: EngineEventRecord) -> SqlResult<()> {
        self.with_connection(|conn| {
            self.ensure_schema(conn)?;
            conn.execute(
                "INSERT INTO engine_events (
                    session_id, objective, kind, detail, payload, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    event.session_id,
                    event.objective,
                    event.kind,
                    event.detail,
                    serde_json::to_string(&event.payload).unwrap_or_else(|_| "null".to_string()),
                    event.created_at,
                ],
            )?;
            Ok(())
        })
    }

    pub async fn records_for_objective(&self, objective: &str) -> SqlResult<Vec<EvidenceRecord>> {
        self.with_connection(|conn| {
            self.ensure_schema(conn)?;
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
            self.ensure_schema(conn)?;
            let mut stmt = conn.prepare(
                "SELECT id, objective, task_id, claim, source, provider, confidence, contradictions, notes, payload, generated_at
                 FROM evidence_records
                 ORDER BY generated_at ASC, id ASC",
            )?;
            let rows = stmt.query_map([], |row| row_to_record(row))?;
            rows.collect()
        })
    }

    pub async fn session_memory(&self, session_id: &str) -> SqlResult<Vec<SessionMemoryRecord>> {
        self.with_connection(|conn| {
            self.ensure_schema(conn)?;
            let mut stmt = conn.prepare(
                "SELECT session_id, objective, key, value, created_at
                 FROM session_memory
                 WHERE session_id = ?1
                 ORDER BY created_at ASC, key ASC",
            )?;
            let rows = stmt.query_map(params![session_id], |row| {
                Ok(SessionMemoryRecord {
                    session_id: row.get(0)?,
                    objective: row.get(1)?,
                    key: row.get(2)?,
                    value: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(Value::Null),
                    created_at: row.get(4)?,
                })
            })?;
            rows.collect()
        })
    }

    pub async fn engine_events(&self, session_id: &str) -> SqlResult<Vec<EngineEventRecord>> {
        self.with_connection(|conn| {
            self.ensure_schema(conn)?;
            let mut stmt = conn.prepare(
                "SELECT session_id, objective, kind, detail, payload, created_at
                 FROM engine_events
                 WHERE session_id = ?1
                 ORDER BY created_at ASC, rowid ASC",
            )?;
            let rows = stmt.query_map(params![session_id], |row| {
                Ok(EngineEventRecord {
                    session_id: row.get(0)?,
                    objective: row.get(1)?,
                    kind: row.get(2)?,
                    detail: row.get(3)?,
                    payload: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or(Value::Null),
                    created_at: row.get(5)?,
                })
            })?;
            rows.collect()
        })
    }

    fn ensure_schema(&self, conn: &Connection) -> SqlResult<()> {
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
            "CREATE TABLE IF NOT EXISTS session_memory (
                session_id TEXT NOT NULL,
                objective TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (session_id, key)
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS engine_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                objective TEXT NOT NULL,
                kind TEXT NOT NULL,
                detail TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
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
