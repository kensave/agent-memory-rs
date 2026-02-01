use agent_memory_rs::{MemorySystem, ModelType};
use agent_memory_rs::storage::memory_store::Memory;
use std::fs;

#[test]
fn test_semantic_search_with_different_words() {
    let db_path = "/tmp/test_semantic_search.db";
    let _ = fs::remove_file(db_path);

    let system = MemorySystem::new_with_model(db_path, ModelType::MiniLM).unwrap();

    // Create workspace
    system.database().execute(|conn| {
        conn.execute("INSERT INTO workspaces (name, path) VALUES ('test', '/tmp')", [])?;
        Ok(())
    }).unwrap();

    // Store memory with specific words
    let memory1 = Memory {
        id: None,
        workspace_id: 1,
        agent_id: None,
        text: "I love eating pizza on Friday evenings with pepperoni and mushrooms".to_string(),
        tags: Some("food, preferences".to_string()),
        importance_score: 0.8,
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

    let memory2 = Memory {
        id: None,
        workspace_id: 1,
        agent_id: None,
        text: "TypeScript is my preferred programming language for web development".to_string(),
        tags: Some("programming, preferences".to_string()),
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

    let memory3 = Memory {
        id: None,
        workspace_id: 1,
        agent_id: None,
        text: "The weather is sunny today in California".to_string(),
        tags: Some("weather".to_string()),
        importance_score: 0.3,
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

    system.learn(&memory1).unwrap();
    system.learn(&memory2).unwrap();
    system.learn(&memory3).unwrap();

    // Test 1: Exact keyword match should have high similarity
    let results = system.search("pizza Friday", &Default::default(), 5).unwrap();
    assert!(!results.is_empty(), "Should find results for exact keywords");
    let top_result = &results[0];
    assert!(top_result.memory.text.contains("pizza"), "Top result should contain 'pizza'");
    println!("Test 1 - Exact keywords: similarity={:.3}", top_result.similarity_score);

    // Test 2: Semantic search with different words (same concept)
    let results = system.search("weekend Italian food preferences", &Default::default(), 5).unwrap();
    assert!(!results.is_empty(), "Should find results for semantic query");
    
    // The pizza memory should rank higher than weather
    let pizza_result = results.iter().find(|r| r.memory.text.contains("pizza"));
    let weather_result = results.iter().find(|r| r.memory.text.contains("weather"));
    
    if let (Some(pizza), Some(weather)) = (pizza_result, weather_result) {
        println!("Test 2 - Semantic search:");
        println!("  Pizza similarity: {:.3}", pizza.similarity_score);
        println!("  Weather similarity: {:.3}", weather.similarity_score);
        
        assert!(
            pizza.similarity_score > weather.similarity_score,
            "Pizza (food-related) should have higher similarity than weather for 'Italian food' query"
        );
        
        // Similarity should NOT be 1.0 for semantic matches
        assert!(
            pizza.similarity_score < 1.0,
            "Semantic match should have similarity < 1.0, got {:.3}",
            pizza.similarity_score
        );
    } else {
        panic!("Should find both pizza and weather memories");
    }

    // Test 3: Completely unrelated query
    let results = system.search("machine learning algorithms", &Default::default(), 5).unwrap();
    if !results.is_empty() {
        let top_similarity = results[0].similarity_score;
        println!("Test 3 - Unrelated query: top similarity={:.3}", top_similarity);
        
        // Unrelated queries should have lower similarity
        assert!(
            top_similarity < 0.8,
            "Unrelated query should have low similarity, got {:.3}",
            top_similarity
        );
    }

    println!("\n✅ Semantic search test passed!");
    println!("   - Exact matches work");
    println!("   - Semantic similarity works (different words, same concept)");
    println!("   - Similarity scores vary appropriately");
}
