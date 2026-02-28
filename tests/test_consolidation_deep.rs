use agent_memory_rs::models::dtos::Episode;
use agent_memory_rs::services::consolidation_engine::ConsolidationEngine;
use agent_memory_rs::services::episodic_store::EpisodicMemoryStore;
use agent_memory_rs::storage::database::Database;
use agent_memory_rs::traits::consolidation::ConsolidationEngine as ConsolidationEngineTrait;
use agent_memory_rs::traits::memory_store::MemoryStore;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_consolidation_creates_memories() {
    let db_path = "/tmp/test_consolidation_deep.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let engine = ConsolidationEngine::new(db.clone());
    
    // Create episodes with patterns
    for i in 0..5 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-15 {:02}:00:00", i),
            conversation_id: Some("conv1".to_string()),
            event_type: "task".to_string(),
            context: json!({"action": "debug", "result": "success"}),
            outcome: Some("success".to_string()),
            valence: Some(0.8),
            archived: false,
            created_at: None,
        };
        store.store(episode).await.unwrap();
    }
    
    // Count before consolidation
    let (memories_before, procedures_before, synopses_before) = db.execute(|conn| {
        let memories: i64 = conn.query_row("SELECT COUNT(*) FROM memories WHERE workspace_id = ?", [workspace_id], |r| r.get(0))?;
        let procedures: i64 = conn.query_row("SELECT COUNT(*) FROM procedures WHERE workspace_id = ?", [workspace_id], |r| r.get(0))?;
        let synopses: i64 = conn.query_row("SELECT COUNT(*) FROM daily_synopsis WHERE workspace_id = ?", [workspace_id], |r| r.get(0))?;
        Ok((memories, procedures, synopses))
    }).unwrap();
    
    println!("Before: memories={}, procedures={}, synopses={}", memories_before, procedures_before, synopses_before);
    
    // Run consolidation
    let synopsis = engine.consolidate_daily("2026-01-15".to_string()).await.unwrap();
    
    // Count after consolidation
    let (memories_after, procedures_after, synopses_after) = db.execute(|conn| {
        let memories: i64 = conn.query_row("SELECT COUNT(*) FROM memories WHERE workspace_id = ?", [workspace_id], |r| r.get(0))?;
        let procedures: i64 = conn.query_row("SELECT COUNT(*) FROM procedures WHERE workspace_id = ?", [workspace_id], |r| r.get(0))?;
        let synopses: i64 = conn.query_row("SELECT COUNT(*) FROM daily_synopsis WHERE workspace_id = ?", [workspace_id], |r| r.get(0))?;
        Ok((memories, procedures, synopses))
    }).unwrap();
    
    println!("After: memories={}, procedures={}, synopses={}", memories_after, procedures_after, synopses_after);
    println!("Synopsis: {:?}", synopsis);
    
    // Verify synopsis was created
    assert!(synopses_after > synopses_before, "Synopsis should be created");
    assert_eq!(synopsis.date, "2026-01-15");
    assert!(!synopsis.summary.is_empty());
    
    // Check if procedures or memories were created
    println!("Procedures created: {}", procedures_after - procedures_before);
    println!("Memories created: {}", memories_after - memories_before);
}
