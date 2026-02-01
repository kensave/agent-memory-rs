use agent_memory_rs::models::dtos::Episode;
use agent_memory_rs::services::context_injection::ContextInjectionService;
use agent_memory_rs::services::health_monitor::HealthMonitor;
use agent_memory_rs::services::memory_manager::MemoryManager;
use agent_memory_rs::storage::database::Database;
use agent_memory_rs::storage::memory_store::Memory;
use agent_memory_rs::traits::memory_store::MemoryStore;
use serde_json::json;
use std::fs;

#[test]
fn test_context_injection() {
    let db_path = "/tmp/test_context_injection.db";
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
        text: "Test memory for context".to_string(),
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

    let service = ContextInjectionService::new(manager);
    let context = service.prepare_context("test", workspace_id, 1000).unwrap();
    assert!(!context.is_empty());
}

#[test]
fn test_health_monitor() {
    let db_path = "/tmp/test_health_monitor.db";
    let _ = fs::remove_file(db_path);

    let db = Database::new(db_path).unwrap();
    let workspace_id = db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES (?, ?)", ["test", "/tmp"])?;
        Ok(conn.last_insert_rowid())
    }).unwrap();

    let manager = MemoryManager::new(db);
    let monitor = HealthMonitor::new(manager);
    
    let metrics = monitor.calculate_metrics(workspace_id).unwrap();
    assert!(metrics.health_score >= 0.0 && metrics.health_score <= 1.0);
    
    let health = monitor.check_health(workspace_id).unwrap();
    assert!(!health.is_empty());
}
