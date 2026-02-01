use memory_rs::storage::{Database, Memory, MemoryStore};
use std::fs;

#[test]
fn test_track_source_episode() {
    let db_path = "/tmp/test_semantic_source.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = MemoryStore::new(db);
    
    let memory = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Test knowledge".to_string(),
        tags: None,
        importance_score: 0.5,
        access_count: 0,
        last_accessed: None,
        conversation_id: None,
        parent_memory_id: None,
        user_feedback: None,
        source_episodes: vec![],
        confidence: 0.5,
        last_validated: None,
        created_at: None,
        updated_at: None,
    };
    
    let id = store.insert_memory(&memory).unwrap();
    
    store.track_source_episode(id, 1).unwrap();
    store.track_source_episode(id, 2).unwrap();
    
    let retrieved = store.get_memory(id).unwrap().unwrap();
    assert_eq!(retrieved.source_episodes, vec![1, 2]);
}

#[test]
fn test_update_confidence() {
    let db_path = "/tmp/test_semantic_confidence.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = MemoryStore::new(db);
    
    let memory = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Test knowledge".to_string(),
        tags: None,
        importance_score: 0.5,
        access_count: 0,
        last_accessed: None,
        conversation_id: None,
        parent_memory_id: None,
        user_feedback: None,
        source_episodes: vec![],
        confidence: 0.5,
        last_validated: None,
        created_at: None,
        updated_at: None,
    };
    
    let id = store.insert_memory(&memory).unwrap();
    
    store.update_confidence(id, 0.2).unwrap();
    let retrieved = store.get_memory(id).unwrap().unwrap();
    assert_eq!(retrieved.confidence, 0.7);
    
    store.update_confidence(id, 0.5).unwrap();
    let retrieved = store.get_memory(id).unwrap().unwrap();
    assert_eq!(retrieved.confidence, 1.0);
    
    store.update_confidence(id, -2.0).unwrap();
    let retrieved = store.get_memory(id).unwrap().unwrap();
    assert_eq!(retrieved.confidence, 0.0);
}

#[test]
fn test_validate_knowledge() {
    let db_path = "/tmp/test_semantic_validate.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = MemoryStore::new(db);
    
    let memory = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Test knowledge".to_string(),
        tags: None,
        importance_score: 0.5,
        access_count: 0,
        last_accessed: None,
        conversation_id: None,
        parent_memory_id: None,
        user_feedback: None,
        source_episodes: vec![],
        confidence: 0.5,
        last_validated: None,
        created_at: None,
        updated_at: None,
    };
    
    let id = store.insert_memory(&memory).unwrap();
    
    store.validate_knowledge(id).unwrap();
    
    let retrieved = store.get_memory(id).unwrap().unwrap();
    assert!(retrieved.last_validated.is_some());
}

#[test]
fn test_get_by_confidence_threshold() {
    let db_path = "/tmp/test_semantic_threshold.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = MemoryStore::new(db.clone());
    
    let memory1 = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "High confidence".to_string(),
        tags: None,
        importance_score: 0.5,
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
    
    let memory2 = Memory {
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
        confidence: 0.3,
        last_validated: None,
        created_at: None,
        updated_at: None,
    };
    
    store.insert_memory(&memory1).unwrap();
    store.insert_memory(&memory2).unwrap();
    
    let results = store.get_by_confidence_threshold(workspace_id, 0.7).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].text, "High confidence");
}
