use memory_rs::models::dtos::Episode;
use memory_rs::services::decay_manager::DecayManager;
use memory_rs::services::episodic_store::EpisodicMemoryStore;
use memory_rs::services::procedural_store::ProceduralMemoryStore;
use memory_rs::storage::database::Database;
use memory_rs::storage::memory_store::{Memory, MemoryStore};
use memory_rs::traits::decay::DecayManager as DecayManagerTrait;
use memory_rs::traits::memory_store::MemoryStore as MemoryStoreTrait;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_archive_episodes() {
    let db_path = "/tmp/test_decay_archive.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let manager = DecayManager::new(db);
    
    // Create old episode
    let old_episode = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: "2025-01-01 00:00:00".to_string(),
        conversation_id: None,
        event_type: "test".to_string(),
        context: json!({}),
        outcome: None,
        valence: None,
        archived: false,
        created_at: Some("2025-01-01 00:00:00".to_string()),
    };
    
    store.store(old_episode).await.unwrap();
    
    let archived = manager.archive_episodes(workspace_id, 0.5, false).await.unwrap();
    assert_eq!(archived.len(), 1);
}

#[tokio::test]
async fn test_prune_low_confidence() {
    let db_path = "/tmp/test_decay_prune.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let mem_store = MemoryStore::new(db.clone());
    let manager = DecayManager::new(db);
    
    let memory = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Low confidence".to_string(),
        tags: None,
        importance_score: 0.5,
        access_count: 0,
        last_accessed: None,
        conversation_id: None,
        parent_memory_id: None,
        user_feedback: None,
        source_episodes: vec![],
        confidence: 0.2,
        last_validated: None,
        created_at: None,
        updated_at: None,
    };
    
    mem_store.insert_memory(&memory).unwrap();
    
    let pruned = manager.prune_low_confidence(workspace_id, 0.5, false).await.unwrap();
    assert_eq!(pruned.len(), 1);
}

#[tokio::test]
async fn test_remove_inactive_procedures() {
    let db_path = "/tmp/test_decay_procedures.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let proc_store = ProceduralMemoryStore::new(db.clone());
    let manager = DecayManager::new(db);
    
    let procedure = memory_rs::models::dtos::Procedure {
        id: None,
        workspace_id,
        name: "Old procedure".to_string(),
        trigger_conditions: json!({}),
        action_sequence: json!([]),
        success_rate: 0.5,
        usage_count: 0,
        last_used: Some("2025-01-01 00:00:00".to_string()),
        learned_from: vec![],
        created_at: None,
    };
    
    proc_store.store(procedure).await.unwrap();
    
    let removed = manager.remove_inactive_procedures(workspace_id, 30, false).await.unwrap();
    assert_eq!(removed.len(), 1);
}

#[tokio::test]
async fn test_dry_run_mode() {
    let db_path = "/tmp/test_decay_dryrun.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = EpisodicMemoryStore::new(db.clone());
    let manager = DecayManager::new(db.clone());
    
    let old_episode = Episode {
        id: None,
        workspace_id,
        agent_id: None,
        timestamp: "2025-01-01 00:00:00".to_string(),
        conversation_id: None,
        event_type: "test".to_string(),
        context: json!({}),
        outcome: None,
        valence: None,
        archived: false,
        created_at: Some("2025-01-01 00:00:00".to_string()),
    };
    
    let id = store.store(old_episode).await.unwrap();
    
    let archived = manager.archive_episodes(workspace_id, 0.5, true).await.unwrap();
    assert_eq!(archived.len(), 1);
    
    let episode = store.get(id).await.unwrap().unwrap();
    assert!(!episode.archived, "Dry run should not actually archive");
}
