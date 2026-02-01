use memory_rs::models::dtos::Episode;
use memory_rs::services::episodic_store::EpisodicMemoryStore;
use memory_rs::services::pattern_extractor::PatternExtractor;
use memory_rs::storage::database::Database;
use memory_rs::traits::memory_store::MemoryStore;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_extract_recurring_patterns() {
    let db_path = "/tmp/test_pattern_recurring.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let extractor = PatternExtractor::new(db);
    
    for i in 0..3 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-{:02} 00:00:00", i + 1),
            conversation_id: Some("conv1".to_string()),
            event_type: "code_review".to_string(),
            context: json!({}),
            outcome: None,
            valence: None,
            archived: false,
            created_at: None,
        };
        store.store(episode).await.unwrap();
    }
    
    let patterns = extractor.extract_all_patterns(workspace_id).unwrap();
    let recurring = patterns.iter().find(|p| p.pattern_type == "recurring_event");
    assert!(recurring.is_some());
    assert!(recurring.unwrap().frequency >= 3);
}

#[tokio::test]
async fn test_extract_user_preferences() {
    let db_path = "/tmp/test_pattern_preferences.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let extractor = PatternExtractor::new(db);
    
    for i in 0..3 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-{:02} 00:00:00", i + 1),
            conversation_id: None,
            event_type: "task".to_string(),
            context: json!({}),
            outcome: Some("success".to_string()),
            valence: Some(0.8),
            archived: false,
            created_at: None,
        };
        store.store(episode).await.unwrap();
    }
    
    let patterns = extractor.extract_all_patterns(workspace_id).unwrap();
    let preference = patterns.iter().find(|p| p.pattern_type == "user_preference");
    assert!(preference.is_some());
}

#[tokio::test]
async fn test_extract_workflows() {
    let db_path = "/tmp/test_pattern_workflows.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let extractor = PatternExtractor::new(db);
    
    for i in 0..4 {
        let event_type = if i % 2 == 0 { "start" } else { "finish" };
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-01 {:02}:00:00", i),
            conversation_id: Some("conv1".to_string()),
            event_type: event_type.to_string(),
            context: json!({}),
            outcome: None,
            valence: None,
            archived: false,
            created_at: None,
        };
        store.store(episode).await.unwrap();
    }
    
    let patterns = extractor.extract_all_patterns(workspace_id).unwrap();
    let workflow = patterns.iter().find(|p| p.pattern_type == "workflow");
    assert!(workflow.is_some());
}

#[tokio::test]
async fn test_cluster_similar_episodes() {
    let db_path = "/tmp/test_pattern_clusters.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let extractor = PatternExtractor::new(db);
    
    for i in 0..5 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-{:02} 00:00:00", i + 1),
            conversation_id: None,
            event_type: "debug".to_string(),
            context: json!({}),
            outcome: None,
            valence: None,
            archived: false,
            created_at: None,
        };
        store.store(episode).await.unwrap();
    }
    
    let patterns = extractor.cluster_similar_episodes(workspace_id, 0.8).unwrap();
    assert!(!patterns.is_empty());
    assert!(patterns[0].frequency >= 3);
}
