use agent_memory_rs::storage::Database;
use agent_memory_rs::services::EpisodicMemoryStore;
use agent_memory_rs::models::Episode;
use agent_memory_rs::traits::MemoryStore;
use serde_json::json;

#[tokio::test]
async fn test_episodic_store_crud() {
    use std::fs;
    let db_path = "/tmp/test_episodic_crud.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    // Create workspace first
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')", [])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db);
    
    // Create episode
    let episode = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: "2026-01-31T16:00:00Z".to_string(),
        conversation_id: Some("conv_123".to_string()),
        event_type: "user_query".to_string(),
        context: json!({"query": "test query", "response": "test response"}),
        outcome: Some("success".to_string()),
        valence: Some(0.8),
        archived: false,
        created_at: None,
    };
    
    // Store
    let id = store.store(episode.clone()).await.unwrap();
    assert!(id > 0);
    
    // Get
    let retrieved = store.get(id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.event_type, "user_query");
    assert_eq!(retrieved.valence, Some(0.8));
    
    // Update
    let mut updated = retrieved.clone();
    updated.outcome = Some("updated".to_string());
    store.update(id, updated).await.unwrap();
    
    let retrieved = store.get(id).await.unwrap().unwrap();
    assert_eq!(retrieved.outcome, Some("updated".to_string()));
    
    // Delete
    store.delete(id).await.unwrap();
    let retrieved = store.get(id).await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_episodic_store_batch() {
    use std::fs;
    let db_path = "/tmp/test_episodic_batch.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    // Create workspace first
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')", [])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db);
    
    let episodes = vec![
        Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: "2026-01-31T16:00:00Z".to_string(),
            conversation_id: Some("conv_123".to_string()),
            event_type: "user_query".to_string(),
            context: json!({"query": "test 1"}),
            outcome: None,
            valence: None,
            archived: false,
            created_at: None,
        },
        Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: "2026-01-31T16:01:00Z".to_string(),
            conversation_id: Some("conv_123".to_string()),
            event_type: "tool_execution".to_string(),
            context: json!({"tool": "test_tool"}),
            outcome: Some("success".to_string()),
            valence: Some(0.9),
            archived: false,
            created_at: None,
        },
    ];
    
    let ids = store.store_batch(episodes).await.unwrap();
    assert_eq!(ids.len(), 2);
    
    // Verify both stored
    let ep1 = store.get(ids[0]).await.unwrap().unwrap();
    let ep2 = store.get(ids[1]).await.unwrap().unwrap();
    assert_eq!(ep1.event_type, "user_query");
    assert_eq!(ep2.event_type, "tool_execution");
}

#[tokio::test]
async fn test_episodic_get_by_conversation() {
    use std::fs;
    let db_path = "/tmp/test_episodic_conversation.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    // Create workspace first
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')", [])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db);
    
    // Store episodes in same conversation
    for i in 0..3 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-31T16:0{}:00Z", i),
            conversation_id: Some("conv_456".to_string()),
            event_type: format!("event_{}", i),
            context: json!({"index": i}),
            outcome: None,
            valence: None,
            archived: false,
            created_at: None,
        };
        store.store(episode).await.unwrap();
    }
    
    let episodes = store.get_by_conversation("conv_456").unwrap();
    assert_eq!(episodes.len(), 3);
    assert_eq!(episodes[0].event_type, "event_0");
    assert_eq!(episodes[2].event_type, "event_2");
}

#[tokio::test]
async fn test_episodic_archive() {
    use std::fs;
    let db_path = "/tmp/test_episodic_archive.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    // Create workspace first
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES ('test', '/tmp/test')", [])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db);
    
    let episode = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: "2026-01-31T16:00:00Z".to_string(),
        conversation_id: None,
        event_type: "test".to_string(),
        context: json!({}),
        outcome: None,
        valence: None,
        archived: false,
        created_at: None,
    };
    
    let id = store.store(episode).await.unwrap();
    
    // Archive
    store.archive(id).unwrap();
    
    // Verify archived
    let retrieved = store.get(id).await.unwrap().unwrap();
    assert!(retrieved.archived);
}
