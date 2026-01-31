use anyhow::Result;
use rusqlite::Connection;
use rusqlite::ffi::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;
use std::path::Path;

const SCHEMA_VERSION: i32 = 1;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        // Initialize sqlite-vec extension globally
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        
        let conn = Connection::open(path)?;
        
        let mut db = Database { conn };
        db.initialize()?;
        Ok(db)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    fn initialize(&mut self) -> Result<()> {
        self.create_schema()?;
        self.apply_migrations()?;
        Ok(())
    }

    fn create_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

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
            ",
        )?;

        // Create vec0 virtual table for embeddings
        self.conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS vec0 USING vec0(
                memory_id INTEGER PRIMARY KEY,
                embedding FLOAT[384]
            )",
            [],
        )?;

        Ok(())
    }

    fn apply_migrations(&self) -> Result<()> {
        let current_version: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if current_version < SCHEMA_VERSION {
            self.conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [SCHEMA_VERSION],
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_database_initialization() {
        let db_path = "/tmp/test_memory_init.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).expect("Failed to create database");

        // Verify tables exist
        let tables: Vec<String> = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"workspaces".to_string()));
        assert!(tables.contains(&"agents".to_string()));
        assert!(tables.contains(&"memories".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));

        // Verify vec0 virtual table exists
        assert!(tables.contains(&"vec0".to_string()));

        // Verify schema version
        let version: i32 = db
            .conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_sqlite_vec_extension_loaded() {
        let db_path = "/tmp/test_vec_extension.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).expect("Failed to create database");

        // Test vec_version() function exists
        let version: String = db
            .conn
            .query_row("SELECT vec_version()", [], |row| row.get(0))
            .expect("vec_version() should be available");

        assert!(!version.is_empty());

        fs::remove_file(db_path).ok();
    }

    #[test]
    fn test_indexes_created() {
        let db_path = "/tmp/test_indexes.db";
        let _ = fs::remove_file(db_path);

        let db = Database::new(db_path).expect("Failed to create database");

        let indexes: Vec<String> = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(indexes.contains(&"idx_memories_workspace".to_string()));
        assert!(indexes.contains(&"idx_memories_agent".to_string()));
        assert!(indexes.contains(&"idx_memories_importance".to_string()));

        fs::remove_file(db_path).ok();
    }
}
