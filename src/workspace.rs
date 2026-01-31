use anyhow::{anyhow, Result};
use rusqlite::OptionalExtension;
use std::path::{Path, PathBuf};
use std::fs;
use crate::{MemorySystem, ModelType};

pub struct WorkspaceManager {
    base_dir: PathBuf,
    model_type: ModelType,
}

impl WorkspaceManager {
    pub fn new(model_type: ModelType) -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Cannot find home directory"))?;
        let base_dir = home.join(".memory-rs").join("workspaces");
        fs::create_dir_all(&base_dir)?;
        
        Ok(WorkspaceManager { base_dir, model_type })
    }

    pub fn with_base_dir<P: AsRef<Path>>(base_dir: P, model_type: ModelType) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(&base_dir)?;
        Ok(WorkspaceManager { base_dir, model_type })
    }

    pub fn get_or_create_workspace(&self, workspace_name: &str) -> Result<MemorySystem> {
        let workspace_path = self.workspace_path(workspace_name);
        let db_path = workspace_path.join("memory.db");
        
        fs::create_dir_all(&workspace_path)?;
        
        let system = MemorySystem::new(&db_path, self.model_type)?;
        
        // Ensure workspace entry exists
        let workspace_id: Option<i64> = system.database().connection()
            .query_row(
                "SELECT id FROM workspaces WHERE name = ?1",
                [workspace_name],
                |row| row.get(0),
            )
            .optional()?;
        
        if workspace_id.is_none() {
            system.database().connection().execute(
                "INSERT INTO workspaces (name, path) VALUES (?1, ?2)",
                [workspace_name, workspace_path.to_str().unwrap()],
            )?;
        }
        
        Ok(system)
    }

    pub fn list_workspaces(&self) -> Result<Vec<String>> {
        let mut workspaces = Vec::new();
        
        if !self.base_dir.exists() {
            return Ok(workspaces);
        }
        
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    workspaces.push(name.to_string());
                }
            }
        }
        
        workspaces.sort();
        Ok(workspaces)
    }

    pub fn delete_workspace(&self, workspace_name: &str) -> Result<()> {
        let workspace_path = self.workspace_path(workspace_name);
        if workspace_path.exists() {
            fs::remove_dir_all(workspace_path)?;
        }
        Ok(())
    }

    pub fn workspace_exists(&self, workspace_name: &str) -> bool {
        self.workspace_path(workspace_name).exists()
    }

    fn workspace_path(&self, workspace_name: &str) -> PathBuf {
        self.base_dir.join(workspace_name)
    }

    pub fn detect_workspace_from_cwd() -> Option<String> {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_workspace_creation() {
        let base_dir = "/tmp/test_workspace_mgr";
        let _ = fs::remove_dir_all(base_dir);

        let manager = WorkspaceManager::with_base_dir(base_dir, ModelType::MiniLM).unwrap();
        let system = manager.get_or_create_workspace("test-ws").unwrap();

        // Verify workspace exists
        assert!(manager.workspace_exists("test-ws"));

        // Verify workspace entry in database
        let count: i64 = system.database().connection()
            .query_row("SELECT COUNT(*) FROM workspaces WHERE name = 'test-ws'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        fs::remove_dir_all(base_dir).ok();
    }

    #[test]
    fn test_list_workspaces() {
        let base_dir = "/tmp/test_list_workspaces";
        let _ = fs::remove_dir_all(base_dir);

        let manager = WorkspaceManager::with_base_dir(base_dir, ModelType::MiniLM).unwrap();

        manager.get_or_create_workspace("ws1").unwrap();
        manager.get_or_create_workspace("ws2").unwrap();
        manager.get_or_create_workspace("ws3").unwrap();

        let workspaces = manager.list_workspaces().unwrap();
        assert_eq!(workspaces.len(), 3);
        assert!(workspaces.contains(&"ws1".to_string()));
        assert!(workspaces.contains(&"ws2".to_string()));
        assert!(workspaces.contains(&"ws3".to_string()));

        fs::remove_dir_all(base_dir).ok();
    }

    #[test]
    fn test_workspace_isolation() {
        let base_dir = "/tmp/test_workspace_isolation";
        let _ = fs::remove_dir_all(base_dir);

        let manager = WorkspaceManager::with_base_dir(base_dir, ModelType::MiniLM).unwrap();

        let system1 = manager.get_or_create_workspace("ws1").unwrap();
        let system2 = manager.get_or_create_workspace("ws2").unwrap();

        // Get workspace IDs
        let ws1_id: i64 = system1.database().connection()
            .query_row("SELECT id FROM workspaces WHERE name = 'ws1'", [], |row| row.get(0))
            .unwrap();
        let ws2_id: i64 = system2.database().connection()
            .query_row("SELECT id FROM workspaces WHERE name = 'ws2'", [], |row| row.get(0))
            .unwrap();

        // Learn in workspace 1
        let memory1 = crate::storage::Memory {
            id: None,
            workspace_id: ws1_id,
            agent_id: None,
            text: "WS1 memory".to_string(),
            tags: None,
            importance_score: 0.5,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };
        system1.learn(&memory1).unwrap();

        // Learn in workspace 2
        let memory2 = crate::storage::Memory {
            id: None,
            workspace_id: ws2_id,
            agent_id: None,
            text: "WS2 memory".to_string(),
            tags: None,
            importance_score: 0.5,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            created_at: None,
            updated_at: None,
        };
        system2.learn(&memory2).unwrap();

        // Verify isolation - each workspace should only see its own memories
        let count1: i64 = system1.database().connection()
            .query_row("SELECT COUNT(*) FROM memories WHERE workspace_id = ?1", [ws1_id], |row| row.get(0))
            .unwrap();
        assert_eq!(count1, 1);

        let count2: i64 = system2.database().connection()
            .query_row("SELECT COUNT(*) FROM memories WHERE workspace_id = ?1", [ws2_id], |row| row.get(0))
            .unwrap();
        assert_eq!(count2, 1);

        fs::remove_dir_all(base_dir).ok();
    }

    #[test]
    fn test_delete_workspace() {
        let base_dir = "/tmp/test_delete_workspace";
        let _ = fs::remove_dir_all(base_dir);

        let manager = WorkspaceManager::with_base_dir(base_dir, ModelType::MiniLM).unwrap();

        manager.get_or_create_workspace("temp-ws").unwrap();
        assert!(manager.workspace_exists("temp-ws"));

        manager.delete_workspace("temp-ws").unwrap();
        assert!(!manager.workspace_exists("temp-ws"));

        fs::remove_dir_all(base_dir).ok();
    }
}
