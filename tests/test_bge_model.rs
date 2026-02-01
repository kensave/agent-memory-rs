use agent_memory_rs::{MemorySystem, ModelType};
use agent_memory_rs::storage::memory_store::Memory;
use std::fs;

#[test]
fn test_bge_small_semantic_search() {
    let db_path = "/tmp/test_bge_semantic.db";
    let _ = fs::remove_file(db_path);

    println!("Loading BGE-Small model...");
    let system = MemorySystem::new(db_path, ModelType::BgeSmall).unwrap();

    // Create workspace
    system.database().execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES ('test', '/tmp')", [])?;
        Ok(())
    }).unwrap();

    // Store diverse memories
    let memories = vec![
        ("I enjoy hiking in the mountains on weekends", "outdoor, hobbies"),
        ("Python is great for data science and machine learning", "programming, tech"),
        ("The sunset was beautiful yesterday evening", "nature, observation"),
    ];

    for (text, tags) in memories {
        let memory = Memory {
            id: None,
            workspace_id: 1,
            agent_id: None,
            text: text.to_string(),
            tags: Some(tags.to_string()),
            importance_score: 0.7,
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
        system.learn(&memory).unwrap();
    }

    // Test semantic search
    let results = system.search("outdoor activities and nature", &Default::default(), 5).unwrap();
    assert!(!results.is_empty(), "Should find results");

    println!("\nBGE-Small Search Results for 'outdoor activities and nature':");
    for (i, result) in results.iter().enumerate() {
        println!("  {}. [score={:.3}] {}", i+1, result.similarity_score, result.memory.text);
    }

    // Hiking should rank higher than Python for "outdoor activities"
    let hiking_result = results.iter().find(|r| r.memory.text.contains("hiking"));
    let python_result = results.iter().find(|r| r.memory.text.contains("Python"));

    if let (Some(hiking), Some(python)) = (hiking_result, python_result) {
        assert!(
            hiking.similarity_score > python.similarity_score,
            "Hiking should be more relevant than Python for outdoor query. Got hiking={:.3}, python={:.3}",
            hiking.similarity_score,
            python.similarity_score
        );
    }

    // Verify scores are not all 1.0 (real embeddings)
    let top_score = results[0].similarity_score;
    assert!(top_score < 1.0, "Should not have perfect similarity (got {:.3})", top_score);
    assert!(top_score > 0.0, "Should have positive similarity (got {:.3})", top_score);

    println!("\n✅ BGE-Small semantic search working correctly!");
}
