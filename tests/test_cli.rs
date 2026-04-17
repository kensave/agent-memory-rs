use agent_memory_rs::cli::memory_commands::MemoryCLI;
use agent_memory_rs::models::dtos::Episode;
use agent_memory_rs::services::memory_manager::MemoryManager;
use agent_memory_rs::storage::database::Database;
use serde_json::json;
use std::fs;

#[tokio::test]
async fn test_cli_stats() {
    let db_path = "/tmp/test_cli_stats.db";
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

    let cli = MemoryCLI::new(db_path).unwrap();
    let result = cli.stats(workspace_id);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_query() {
    let db_path = "/tmp/test_cli_query.db";
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
    let memory = agent_memory_rs::storage::memory_store::Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Test memory content".to_string(),
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

    let cli = MemoryCLI::new(db_path).unwrap();
    let result = cli.query(workspace_id, "test", 10);
    assert!(result.is_ok());
}
