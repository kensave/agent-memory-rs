use anyhow::Result;
use rusqlite::Connection;
use rusqlite::ffi::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Thread-safe database wrapper with connection pooling
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Initialize sqlite-vec extension globally
        #[allow(clippy::missing_transmute_annotations)]
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        
        let conn = Connection::open(path)?;
        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db.initialize()?;
        Ok(db)
    }

    /// Execute a function with exclusive access to the connection
    pub fn execute<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }

    fn initialize(&self) -> Result<()> {
        self.execute(|conn| {
            // Create schema version table
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_version (
                    version INTEGER PRIMARY KEY,
                    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                )"
            )?;
            Ok(())
        })?;
        
        self.create_schema()?;
        self.apply_migrations()?;
        Ok(())
    }

    fn create_schema(&self) -> Result<()> {
        self.execute(|conn| {
            conn.execute_batch(
                "
                CREATE TABLE IF NOT EXISTS workspaces (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    path TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE IF NOT EXISTS agents (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    workspace_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
                    UNIQUE(workspace_id, name)
                );

                CREATE TABLE IF NOT EXISTS memories (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    workspace_id INTEGER NOT NULL,
                    agent_id INTEGER,
                    text TEXT NOT NULL,
                    source_path TEXT,
                    tags TEXT,
                    importance_score REAL DEFAULT 0.5,
                    access_count INTEGER DEFAULT 0,
                    last_accessed TEXT,
                    conversation_id TEXT,
                    parent_memory_id INTEGER,
                    user_feedback TEXT,
                    source_episodes TEXT DEFAULT '[]',
                    confidence REAL DEFAULT 0.5,
                    last_validated TEXT,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
                    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL,
                    FOREIGN KEY (parent_memory_id) REFERENCES memories(id) ON DELETE SET NULL
                );

                CREATE INDEX IF NOT EXISTS idx_memories_workspace ON memories(workspace_id);
                CREATE INDEX IF NOT EXISTS idx_memories_agent ON memories(agent_id);
                CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance_score);
                CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at);
                CREATE INDEX IF NOT EXISTS idx_memories_conversation ON memories(conversation_id);

                CREATE TABLE IF NOT EXISTS episodes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    workspace_id INTEGER NOT NULL,
                    agent_id INTEGER,
                    timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    conversation_id TEXT,
                    event_type TEXT NOT NULL,
                    context TEXT NOT NULL,
                    outcome TEXT,
                    valence REAL CHECK (valence IS NULL OR (valence >= -1.0 AND valence <= 1.0)),
                    archived INTEGER DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
                    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL
                );

                CREATE INDEX IF NOT EXISTS idx_episodes_workspace ON episodes(workspace_id);
                CREATE INDEX IF NOT EXISTS idx_episodes_timestamp ON episodes(timestamp DESC);
                CREATE INDEX IF NOT EXISTS idx_episodes_conversation ON episodes(conversation_id);
                CREATE INDEX IF NOT EXISTS idx_episodes_archived ON episodes(archived) WHERE archived = 0;

                CREATE TABLE IF NOT EXISTS procedures (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    workspace_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    trigger_conditions TEXT NOT NULL,
                    action_sequence TEXT NOT NULL,
                    success_rate REAL DEFAULT 0.0,
                    usage_count INTEGER DEFAULT 0,
                    last_used TEXT,
                    learned_from TEXT DEFAULT '[]',
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
                );

                CREATE INDEX IF NOT EXISTS idx_procedures_workspace ON procedures(workspace_id);
                CREATE INDEX IF NOT EXISTS idx_procedures_name ON procedures(name);
                CREATE INDEX IF NOT EXISTS idx_procedures_last_used ON procedures(last_used DESC);

                CREATE TABLE IF NOT EXISTS daily_synopsis (
                    date TEXT NOT NULL,
                    workspace_id INTEGER NOT NULL,
                    agent_id INTEGER,
                    summary TEXT NOT NULL,
                    key_insights TEXT DEFAULT '[]',
                    new_knowledge_ids TEXT DEFAULT '[]',
                    new_procedure_ids TEXT DEFAULT '[]',
                    stats TEXT DEFAULT '{}',
                    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (date, workspace_id, agent_id),
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
                    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE SET NULL
                );

                CREATE INDEX IF NOT EXISTS idx_synopsis_date ON daily_synopsis(date DESC);
                CREATE INDEX IF NOT EXISTS idx_synopsis_workspace ON daily_synopsis(workspace_id);
                "
            )?;

            // Create vector tables
            conn.execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS vec0 USING vec0(
                    memory_id INTEGER PRIMARY KEY,
                    embedding FLOAT[384]
                )",
                [],
            )?;

            conn.execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS vec_episodes USING vec0(
                    episode_id INTEGER PRIMARY KEY,
                    embedding FLOAT[384]
                )",
                [],
            )?;

            conn.execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS vec_procedures USING vec0(
                    procedure_id INTEGER PRIMARY KEY,
                    embedding FLOAT[384]
                )",
                [],
            )?;

            conn.execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS vec_synopsis USING vec0(
                    synopsis_id INTEGER PRIMARY KEY,
                    embedding FLOAT[384]
                )",
                [],
            )?;

            Ok(())
        })
    }

    fn apply_migrations(&self) -> Result<()> {
        self.execute(|conn| {
            let current_version: i32 = conn
                .query_row(
                    "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            const SCHEMA_VERSION: i32 = 1;
            
            if current_version < SCHEMA_VERSION {
                conn.execute(
                    "INSERT INTO schema_version (version) VALUES (?1)",
                    [SCHEMA_VERSION],
                )?;
            }

            Ok(())
        })
    }
}
