use crate::services::memory_manager::MemoryManager;
use crate::storage::database::Database;
use crate::models::dtos::Episode;
use anyhow::Result;

pub struct MemoryCLI {
    manager: MemoryManager,
    db: Database,
}

impl MemoryCLI {
    pub fn new(db_path: &str) -> Result<Self> {
        let db = Database::new(db_path)?;
        let manager = MemoryManager::new(db.clone());
        Ok(Self { manager, db })
    }

    pub fn create_workspace(&self, name: &str, path: &str) -> Result<i64> {
        println!("🏗️  Creating workspace '{}'...", name);
        let workspace_id = self.db.execute(|conn| {
            conn.execute(
                "INSERT INTO workspaces (name, path) VALUES (?1, ?2)",
                [name, path],
            )?;
            Ok(conn.last_insert_rowid())
        })?;
        println!("✅ Workspace created with ID: {}", workspace_id);
        Ok(workspace_id)
    }

    pub fn list_workspaces(&self) -> Result<()> {
        println!("📋 Workspaces");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let workspaces = self.db.execute(|conn| {
            let mut stmt = conn.prepare("SELECT id, name, path, created_at FROM workspaces")?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?;
            let result: Vec<_> = rows.collect::<Result<Vec<_>, rusqlite::Error>>()?;
            Ok(result)
        })?;

        if workspaces.is_empty() {
            println!("No workspaces found.");
        } else {
            for (id, name, path, created_at) in workspaces {
                println!("\n  ID: {}", id);
                println!("  Name: {}", name);
                println!("  Path: {}", path);
                println!("  Created: {}", created_at);
            }
        }
        Ok(())
    }

    pub fn delete_workspace(&self, workspace_id: i64) -> Result<()> {
        println!("🗑️  Deleting workspace {}...", workspace_id);
        
        let deleted = self.db.execute(|conn| {
            let rows = conn.execute("DELETE FROM workspaces WHERE id = ?1", [workspace_id])?;
            Ok(rows)
        })?;
        
        if deleted > 0 {
            println!("✅ Workspace {} deleted (CASCADE: memories, episodes, procedures)", workspace_id);
        } else {
            println!("❌ Workspace {} not found", workspace_id);
        }
        Ok(())
    }

    pub fn stats(&self, workspace_id: i64) -> Result<()> {
        println!("📊 Memory Statistics");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let stats = self.manager.get_memory_stats(workspace_id)?;
        println!("  Active Episodes:   {}", stats.active_episodes);
        println!("  Archived Episodes: {}", stats.archived_episodes);
        println!("  Knowledge Items:   {}", stats.knowledge_count);
        
        let total = stats.active_episodes + stats.archived_episodes;
        if total > 0 {
            let health = (stats.active_episodes as f64 / total as f64) * 100.0;
            println!("\n  Memory Health:     {:.1}%", health);
        }
        Ok(())
    }


    pub fn query(&self, workspace_id: i64, query: &str, limit: usize) -> Result<()> {
        println!("🔍 Searching for: \"{}\"", query);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let results = self.manager.retrieve(query, workspace_id, limit)?;
        
        if results.is_empty() {
            println!("No results found.");
        } else {
            for (i, result) in results.iter().enumerate() {
                println!("\n{}. [{}] Score: {:.3}", 
                    i + 1, result.memory_type, result.score);
                println!("   {}", result.content);
            }
        }
        Ok(())
    }

    pub async fn store_episode(&self, workspace_id: i64, event_type: &str, context: &str, outcome: Option<&str>, valence: Option<f64>) -> Result<()> {
        println!("💾 Storing episode...");
        
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: chrono::Local::now().to_rfc3339(),
            conversation_id: None,
            event_type: event_type.to_string(),
            context: serde_json::json!({ "text": context }),
            outcome: outcome.map(|s| s.to_string()),
            valence,
            archived: false,
            created_at: None,
        };
        
        let episode_id = self.manager.store_episode(episode).await?;
        
        println!("✅ Episode stored with ID: {}", episode_id);
        Ok(())
    }
}
