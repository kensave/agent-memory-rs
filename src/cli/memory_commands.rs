use crate::services::memory_manager::MemoryManager;
use crate::storage::database::Database;
use anyhow::Result;

pub struct MemoryCLI {
    manager: MemoryManager,
}

impl MemoryCLI {
    pub fn new(db_path: &str) -> Result<Self> {
        let db = Database::new(db_path)?;
        Ok(Self {
            manager: MemoryManager::new(db),
        })
    }

    pub async fn consolidate(&self, date: String) -> Result<()> {
        println!("🔄 Consolidating memories for {}...", date);
        let synopsis = self.manager.consolidate(date).await?;
        println!("✅ Consolidation complete!");
        println!("   Summary: {}", synopsis.summary);
        println!("   Insights: {}", synopsis.key_insights.len());
        println!("   New knowledge: {}", synopsis.new_knowledge_ids.len());
        println!("   New procedures: {}", synopsis.new_procedure_ids.len());
        Ok(())
    }

    pub fn synopsis(&self, workspace_id: i64, date: &str) -> Result<()> {
        println!("📋 Daily Synopsis for {}", date);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        match self.manager.get_synopsis(workspace_id, date)? {
            Some(synopsis) => {
                println!("\n{}", synopsis.summary);
                println!("\n🔑 Key Insights:");
                for (i, insight) in synopsis.key_insights.iter().enumerate() {
                    println!("  {}. {}", i + 1, insight);
                }
                println!("\n📊 Stats: {}", synopsis.stats);
            }
            None => {
                println!("No synopsis found for this date.");
            }
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
        println!("  Procedures:        {}", stats.procedure_count);
        
        let total = stats.active_episodes + stats.archived_episodes;
        if total > 0 {
            let health = (stats.active_episodes as f64 / total as f64) * 100.0;
            println!("\n  Memory Health:     {:.1}%", health);
        }
        Ok(())
    }

    pub async fn prune(&self, workspace_id: i64, dry_run: bool) -> Result<()> {
        if dry_run {
            println!("🔍 Dry-run mode: No changes will be made");
        } else {
            println!("⚠️  Pruning memories...");
        }
        
        let (episodes, knowledge, procedures) = self.manager.prune(workspace_id, dry_run).await?;
        
        println!("✅ Prune complete!");
        println!("   Episodes archived:  {}", episodes);
        println!("   Knowledge pruned:   {}", knowledge);
        println!("   Procedures removed: {}", procedures);
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
}
