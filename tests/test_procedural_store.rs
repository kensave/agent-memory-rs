use agent_memory_rs::models::dtos::Procedure;
use agent_memory_rs::services::procedural_store::ProceduralMemoryStore;
use agent_memory_rs::storage::database::Database;
use agent_memory_rs::traits::memory_store::MemoryStore;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_procedural_store_crud() {
    let db_path = "/tmp/test_procedural_crud.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = ProceduralMemoryStore::new(db);
    
    let procedure = Procedure {
        id: None,
        workspace_id,
        name: "Deploy workflow".to_string(),
        trigger_conditions: json!({"event": "push", "branch": "main"}),
        action_sequence: json!(["build", "test", "deploy"]),
        success_rate: 0.95,
        usage_count: 10,
        last_used: None,
        learned_from: vec![1, 2, 3],
        created_at: None,
    };
    
    let id = store.store(procedure.clone()).await.unwrap();
    assert!(id > 0);
    
    let retrieved = store.get(id).await.unwrap().unwrap();
    assert_eq!(retrieved.name, "Deploy workflow");
    assert_eq!(retrieved.success_rate, 0.95);
    
    let mut updated = retrieved.clone();
    updated.success_rate = 0.98;
    store.update(id, updated).await.unwrap();
    
    let retrieved = store.get(id).await.unwrap().unwrap();
    assert_eq!(retrieved.success_rate, 0.98);
    
    store.delete(id).await.unwrap();
    assert!(store.get(id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_get_by_trigger() {
    let db_path = "/tmp/test_procedural_trigger.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = ProceduralMemoryStore::new(db);
    
    store.store(Procedure {
        id: None,
        workspace_id,
        name: "Deploy on push".to_string(),
        trigger_conditions: json!({"event": "push"}),
        action_sequence: json!(["deploy"]),
        success_rate: 0.9,
        usage_count: 5,
        last_used: None,
        learned_from: vec![],
        created_at: None,
    }).await.unwrap();
    
    let results = store.get_by_trigger(workspace_id, &json!({"event": "push"})).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Deploy on push");
}

#[tokio::test]
async fn test_update_success_rate() {
    let db_path = "/tmp/test_procedural_success.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = ProceduralMemoryStore::new(db);
    
    let id = store.store(Procedure {
        id: None,
        workspace_id,
        name: "Test workflow".to_string(),
        trigger_conditions: json!({}),
        action_sequence: json!([]),
        success_rate: 0.8,
        usage_count: 10,
        last_used: None,
        learned_from: vec![],
        created_at: None,
    }).await.unwrap();
    
    store.update_success_rate(id, true).unwrap();
    
    let procedure = store.get(id).await.unwrap().unwrap();
    let expected = (0.8 * 10.0 + 1.0) / 11.0;
    assert!((procedure.success_rate - expected).abs() < 0.001);
}

#[tokio::test]
async fn test_increment_usage() {
    let db_path = "/tmp/test_procedural_usage.db";
    let _ = fs::remove_file(db_path);
    
    let db = Database::new(db_path).unwrap();
    
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();
    
    let store = ProceduralMemoryStore::new(db);
    
    let id = store.store(Procedure {
        id: None,
        workspace_id,
        name: "Test workflow".to_string(),
        trigger_conditions: json!({}),
        action_sequence: json!([]),
        success_rate: 0.8,
        usage_count: 5,
        last_used: None,
        learned_from: vec![],
        created_at: None,
    }).await.unwrap();
    
    store.increment_usage(id).unwrap();
    
    let procedure = store.get(id).await.unwrap().unwrap();
    assert_eq!(procedure.usage_count, 6);
    assert!(procedure.last_used.is_some());
}
