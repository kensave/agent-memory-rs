use agent_memory_rs::services::hybrid_retrieval::HybridRetrievalEngine;
use agent_memory_rs::storage::database::Database;
use agent_memory_rs::storage::memory_store::{Memory, MemoryStore};
use std::fs;

#[test]
fn test_bm25_search() {
    let db_path = "/tmp/test_hybrid_bm25.db";
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

    let store = MemoryStore::new(db.clone());
    let engine = HybridRetrievalEngine::new(db);

    let memory = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Rust programming language best practices".to_string(),
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
    store.insert_memory(&memory).unwrap();

    let results = engine
        .search_bm25("Rust programming", workspace_id, 10)
        .unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].memory_type, "semantic");
}

#[test]
fn test_hybrid_search() {
    let db_path = "/tmp/test_hybrid_search.db";
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

    let store = MemoryStore::new(db.clone());
    let engine = HybridRetrievalEngine::new(db);

    for i in 0..3 {
        let memory = Memory {
            id: None,
            workspace_id,
            agent_id: None,
            text: format!("Memory about testing number {}", i),
            tags: None,
            importance_score: 0.7,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            source_episodes: vec![],
            confidence: 0.7,
            last_validated: None,
            created_at: None,
            updated_at: None,
        };
        store.insert_memory(&memory).unwrap();
    }

    let results = engine.hybrid_search("testing", workspace_id, 5).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_search_by_type() {
    let db_path = "/tmp/test_hybrid_by_type.db";
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

    let store = MemoryStore::new(db.clone());
    let engine = HybridRetrievalEngine::new(db);

    let memory = Memory {
        id: None,
        workspace_id,
        agent_id: None,
        text: "Semantic memory content".to_string(),
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
    store.insert_memory(&memory).unwrap();

    let results = engine
        .search_by_type("semantic", workspace_id, "semantic", 10)
        .unwrap();
    assert!(!results.is_empty());
}
