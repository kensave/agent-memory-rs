use agent_memory_rs::models::dtos::Episode;
use agent_memory_rs::services::memory_manager::MemoryManager;
use agent_memory_rs::storage::database::Database;
use agent_memory_rs::storage::memory_store::Memory;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_store_and_retrieve() {
    let db_path = "/tmp/test_manager_store.db";
    let _ = fs::remove_file(db_path);

    let db = Database::new(db_path).unwrap();

    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();

    let manager = MemoryManager::new(db);

    let memory = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Test knowledge about Rust".to_string(),
        tags: None,
        importance_score: 0.8,
        access_count: 0,
        last_accessed: None,
        conversation_id: None,
        parent_memory_id: None,
        user_feedback: None,
        source_episodes: vec![],
        confidence: 0.8,
        last_validated: None,
        created_at: None,
        updated_at: None,
    };
    
    manager.store_knowledge(&memory).unwrap();

    let results = manager.retrieve("Rust", workspace_id, 10).unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_hierarchical_retrieval() {
    let db_path = "/tmp/test_manager_hierarchical.db";
    let _ = fs::remove_file(db_path);

    let db = Database::new(db_path).unwrap();

    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();

    let manager = MemoryManager::new(db);

    let memory = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Programming best practices".to_string(),
        tags: None,
        importance_score: 0.9,
        access_count: 0,
        last_accessed: None,
        conversation_id: None,
        parent_memory_id: None,
        user_feedback: None,
        source_episodes: vec![],
        confidence: 0.9,
        last_validated: None,
        created_at: None,
        updated_at: None,
    };
    
    manager.store_knowledge(&memory).unwrap();

    let episode = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: "2026-01-31 10:00:00".to_string(),
        conversation_id: None,
        event_type: "programming".to_string(),
        context: json!({}),
        outcome: None,
        valence: None,
        archived: false,
        created_at: None,
    };
    manager.store_episode(episode).await.unwrap();

    let results = manager.retrieve_hierarchical("programming", workspace_id, 10).unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_memory_stats() {
    let db_path = "/tmp/test_manager_stats.db";
    let _ = fs::remove_file(db_path);

    let db = Database::new(db_path).unwrap();

    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();

    let manager = MemoryManager::new(db);

    let episode = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: "2026-01-31 10:00:00".to_string(),
        conversation_id: None,
        event_type: "test".to_string(),
        context: json!({}),
        outcome: None,
        valence: None,
        archived: false,
        created_at: None,
    };
    manager.store_episode(episode).await.unwrap();

    let stats = manager.get_memory_stats(workspace_id).unwrap();
    assert_eq!(stats.active_episodes, 1);
    assert_eq!(stats.archived_episodes, 0);
}
