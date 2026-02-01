use agent_memory_rs::models::dtos::Episode;
use agent_memory_rs::services::episodic_store::EpisodicMemoryStore;
use agent_memory_rs::services::synopsis_generator::DailySynopsisGenerator;
use agent_memory_rs::storage::database::Database;
use agent_memory_rs::traits::memory_store::MemoryStore;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_generate_synopsis() {
    let db_path = "/tmp/test_synopsis_generate.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let generator = DailySynopsisGenerator::new(db);
    
    for i in 0..5 {
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
    
    let synopsis = generator.generate_synopsis(workspace_id, "2026-01-15").unwrap();
    assert_eq!(synopsis.date, "2026-01-15");
    assert!(!synopsis.summary.is_empty());
    assert!(!synopsis.key_insights.is_empty());
}

#[tokio::test]
async fn test_store_synopsis() {
    let db_path = "/tmp/test_synopsis_store.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let generator = DailySynopsisGenerator::new(db.clone());
    
    let episode = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: "2026-01-15 10:00:00".to_string(),
        conversation_id: None,
        event_type: "test".to_string(),
        context: json!({}),
        outcome: None,
        valence: None,
        archived: false,
        created_at: None,
    };
    store.store(episode).await.unwrap();
    
    let synopsis = generator.generate_synopsis(workspace_id, "2026-01-15").unwrap();
    let id = generator.store_synopsis(&synopsis).unwrap();
    assert!(id > 0);
}

#[tokio::test]
async fn test_daily_stats() {
    let db_path = "/tmp/test_synopsis_stats.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let generator = DailySynopsisGenerator::new(db);
    
    for i in 0..3 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-15 {:02}:00:00", i),
            conversation_id: None,
            event_type: "task".to_string(),
            context: json!({}),
            outcome: Some("done".to_string()),
            valence: Some(0.9),
            archived: false,
            created_at: None,
        };
        store.store(episode).await.unwrap();
    }
    
    let synopsis = generator.generate_synopsis(workspace_id, "2026-01-15").unwrap();
    let stats = synopsis.stats.as_object().unwrap();
    assert_eq!(stats["total_episodes"], 3);
    assert_eq!(stats["with_outcome"], 3);
    assert_eq!(stats["positive_valence"], 3);
}
