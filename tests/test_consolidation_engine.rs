use memory_rs::models::dtos::Episode;
use memory_rs::services::consolidation_engine::ConsolidationEngine;
use memory_rs::services::episodic_store::EpisodicMemoryStore;
use memory_rs::storage::database::Database;
use memory_rs::traits::consolidation::ConsolidationEngine as ConsolidationEngineTrait;
use memory_rs::traits::memory_store::MemoryStore;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_consolidate_daily() {
    let db_path = "/tmp/test_consolidation.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let engine = ConsolidationEngine::new(db);
    
    for i in 0..3 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-15 {:02}:00:00", i),
            conversation_id: Some("conv1".to_string()),
            event_type: "task".to_string(),
            context: json!({}),
            outcome: Some("success".to_string()),
            valence: Some(0.8),
            archived: false,
            created_at: None,
        };
        store.store(episode).await.unwrap();
    }
    
    let synopsis = engine.consolidate_daily("2026-01-15".to_string()).await.unwrap();
    assert_eq!(synopsis.date, "2026-01-15");
    assert!(!synopsis.summary.is_empty());
}

#[tokio::test]
async fn test_extract_patterns() {
    let db_path = "/tmp/test_consolidation_patterns.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let engine = ConsolidationEngine::new(db);
    
    let mut episode_ids = Vec::new();
    for i in 0..3 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-15 {:02}:00:00", i),
            conversation_id: None,
            event_type: "debug".to_string(),
            context: json!({}),
            outcome: None,
            valence: None,
            archived: false,
            created_at: None,
        };
        let id = store.store(episode).await.unwrap();
        episode_ids.push(id);
    }
    
    let patterns = engine.extract_patterns(episode_ids).await.unwrap();
    assert!(!patterns.is_empty());
}
