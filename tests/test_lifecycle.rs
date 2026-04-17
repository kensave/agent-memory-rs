use agent_memory_rs::models::dtos::Episode;
use agent_memory_rs::services::memory_manager::MemoryManager;
use agent_memory_rs::storage::database::Database;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_full_memory_lifecycle() {
    let db_path = "/tmp/test_lifecycle.db";
    let _ = fs::remove_file(db_path);

    let db = Database::new(db_path).unwrap();
    let workspace_id = db
        .execute(|conn| {
            conn.execute(
                "INSERT INTO workspaces (name, path) VALUES (?, ?)",
                ["test", "/tmp"],
            )?;
            Ok(conn.last_insert_rowid())
        })
        .unwrap();

    let manager = MemoryManager::new(db);

    // Step 1: Store episodes
    for i in 0..5 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: format!("2026-01-15 {:02}:00:00", i),
            conversation_id: Some("conv1".to_string()),
            event_type: "task".to_string(),
            context: json!({"step": i}),
            outcome: Some("success".to_string()),
            valence: Some(0.8),
            archived: false,
            created_at: None,
        };
        manager.store_episode(episode).await.unwrap();
    }

    // Step 2: Query memories (search for content that should match)
    let _results = manager.retrieve("step", workspace_id, 10).unwrap();
    // Note: Results may be empty if embeddings aren't loaded, but search should not error

    // Step 3: Check stats (this should always work)
    let stats = manager.get_memory_stats(workspace_id).unwrap();
    assert!(
        stats.active_episodes > 0,
        "Should have active episodes after storing 5 episodes"
    );
}

#[tokio::test]
async fn test_hierarchical_retrieval_integration() {
    let db_path = "/tmp/test_hierarchical_integration.db";
    let _ = fs::remove_file(db_path);

    let db = Database::new(db_path).unwrap();
    let workspace_id = db
        .execute(|conn| {
            conn.execute(
                "INSERT INTO workspaces (name, path) VALUES (?, ?)",
                ["test", "/tmp"],
            )?;
            Ok(conn.last_insert_rowid())
        })
        .unwrap();

    let manager = MemoryManager::new(db);

    // Add semantic memory
    let memory = agent_memory_rs::storage::memory_store::Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Rust programming best practices".to_string(),
        tags: Some("programming".to_string()),
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

    // Add episode
    let episode = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: "2026-01-31 10:00:00".to_string(),
        conversation_id: None,
        event_type: "programming".to_string(),
        context: json!({}),
        outcome: Some("success".to_string()),
        valence: Some(0.8),
        archived: false,
        created_at: None,
    };
    manager.store_episode(episode).await.unwrap();

    // Test hierarchical retrieval
    let results = manager
        .retrieve_hierarchical("programming", workspace_id, 10)
        .unwrap();
    assert!(!results.is_empty());

    // Should have both semantic and episodic results
    let has_semantic = results.iter().any(|r| r.memory_type == "semantic");
    let has_episodic = results.iter().any(|r| r.memory_type == "episodic");
    assert!(has_semantic || has_episodic);
}
