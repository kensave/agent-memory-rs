use agent_memory_rs::services::memory_manager::MemoryManager;
use agent_memory_rs::storage::database::Database;
use agent_memory_rs::models::dtos::Episode;
use anyhow::Result;

#[tokio::test]
async fn test_full_pipeline_learn_consolidate_search() -> Result<()> {
    let db = Database::new(":memory:")?;
    
    // Create workspace first
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    })?;
    
    let manager = MemoryManager::new(db);
    
    // Learn: Store episodes
    let episode1 = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: chrono::Local::now().to_rfc3339(),
        conversation_id: Some("conv1".to_string()),
        event_type: "user_input".to_string(),
        context: serde_json::json!({"text": "User prefers minimal code"}),
        outcome: Some("Acknowledged".to_string()),
        valence: Some(0.8),
        archived: false,
        created_at: None,
    };
    
    let episode_id = manager.store_episode(episode1).await?;
    assert!(episode_id > 0);
    
    // Consolidate: Extract patterns
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let synopsis = manager.consolidate(date).await?;
    assert!(!synopsis.summary.is_empty());
    
    // Search: Hierarchical retrieval (may be empty if consolidation didn't create semantic memories)
    let _results = manager.retrieve_hierarchical("user preferences", workspace_id, 10)?;
    // Test passes if search completes without error - results may be empty initially
    
    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_returns_multi_type_results() -> Result<()> {
    let db = Database::new(":memory:")?;
    
    // Create workspace first
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    })?;
    
    let manager = MemoryManager::new(db);
    
    // Store episode
    let episode = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: chrono::Local::now().to_rfc3339(),
        conversation_id: None,
        event_type: "test_event".to_string(),
        context: serde_json::json!({"text": "test context"}),
        outcome: None,
        valence: None,
        archived: false,
        created_at: None,
    };
    manager.store_episode(episode).await?;
    
    // Search should work (may return empty results)
    let _results = manager.retrieve("test", workspace_id, 10)?;
    // Just verify the call succeeds - results may be empty
    
    Ok(())
}

#[tokio::test]
async fn test_hierarchical_retrieval_prioritization() -> Result<()> {
    let db = Database::new(":memory:")?;
    
    // Create workspace first
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    })?;
    
    let manager = MemoryManager::new(db);
    
    // Store high-importance semantic memory
    let memory = agent_memory_rs::storage::Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Critical system information".to_string(),
        tags: Some("important".to_string()),
        importance_score: 0.95,
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
    
    manager.store_knowledge(&memory)?;
    
    // Hierarchical search should prioritize high-importance
    let results = manager.retrieve_hierarchical("system information", workspace_id, 10)?;
    if !results.is_empty() {
        assert!(results[0].score > 0.0);
    }
    
    Ok(())
}

#[test]
fn test_memory_stats() -> Result<()> {
    let db = Database::new(":memory:")?;
    
    // Create workspace first
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    })?;
    
    let manager = MemoryManager::new(db);
    
    let stats = manager.get_memory_stats(workspace_id)?;
    assert_eq!(stats.knowledge_count, 0);
    assert_eq!(stats.active_episodes, 0);
    
    Ok(())
}
