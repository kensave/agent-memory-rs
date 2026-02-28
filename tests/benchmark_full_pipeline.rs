use agent_memory_rs::services::memory_manager::MemoryManager;
use agent_memory_rs::storage::database::Database;
use agent_memory_rs::models::dtos::Episode;
use anyhow::Result;
use std::time::Instant;

#[tokio::test]
async fn benchmark_full_pipeline_end_to_end() -> Result<()> {
    println!("\n🚀 Full Memory Pipeline End-to-End Benchmark");
    println!("============================================\n");
    
    let db = Database::new(":memory:")?;
    
    // Create workspace first
    db.execute(|conn| {
        conn.execute(
            "INSERT INTO workspaces (id, name, path, created_at) VALUES (1, 'benchmark', '/tmp/benchmark', datetime('now'))",
            []
        )?;
        Ok(())
    })?;
    
    let manager = MemoryManager::new(db);
    let workspace_id = 1;
    
    // Benchmark 1: Episode Storage
    println!("📝 Benchmark 1: Episode Storage");
    let mut episode_times = Vec::new();
    for i in 0..100 {
        let episode = Episode {
            id: None,
            workspace_id,
            agent_id: None,
            timestamp: chrono::Local::now().to_rfc3339(),
            conversation_id: Some(format!("conv_{}", i % 10)),
            event_type: "user_input".to_string(),
            context: serde_json::json!({
                "text": format!("This is test message number {} with some context", i)
            }),
            outcome: Some("Processed".to_string()),
            valence: Some(0.8),
            archived: false,
            created_at: None,
        };
        
        let start = Instant::now();
        manager.store_episode(episode).await?;
        episode_times.push(start.elapsed());
    }
    
    let avg_episode = episode_times.iter().sum::<std::time::Duration>() / episode_times.len() as u32;
    println!("  ✅ Stored 100 episodes");
    println!("  ⏱️  Average: {:?}", avg_episode);
    println!("  📊 Rate: {:.1} episodes/sec\n", 1.0 / avg_episode.as_secs_f64());
    
    // Benchmark 2: Semantic Memory Storage (high importance)
    println!("📚 Benchmark 2: Semantic Memory Storage");
    let mut semantic_times = Vec::new();
    for i in 0..50 {
        let memory = agent_memory_rs::storage::Memory {
            id: None,
            workspace_id,
            agent_id: None,
            text: format!("Important knowledge item {}: Rust is a systems programming language", i),
            tags: Some("knowledge".to_string()),
            importance_score: 0.85,
            access_count: 0,
            last_accessed: None,
            conversation_id: None,
            parent_memory_id: None,
            user_feedback: None,
            source_episodes: vec![i as i64],
            confidence: 0.9,
            last_validated: None,
            created_at: None,
            updated_at: None,
        };
        
        let start = Instant::now();
        manager.store_knowledge(&memory)?;
        semantic_times.push(start.elapsed());
    }
    
    let avg_semantic = semantic_times.iter().sum::<std::time::Duration>() / semantic_times.len() as u32;
    println!("  ✅ Stored 50 semantic memories");
    println!("  ⏱️  Average: {:?}", avg_semantic);
    println!("  📊 Rate: {:.1} memories/sec\n", 1.0 / avg_semantic.as_secs_f64());
    
    // Benchmark 3: BM25 Search
    println!("🔍 Benchmark 3: BM25 Search");
    let mut bm25_times = Vec::new();
    for _ in 0..20 {
        let start = Instant::now();
        let _results = manager.retrieve("test message", workspace_id, 10)?;
        bm25_times.push(start.elapsed());
    }
    
    let avg_bm25 = bm25_times.iter().sum::<std::time::Duration>() / bm25_times.len() as u32;
    println!("  ✅ Ran 20 searches");
    println!("  ⏱️  Average: {:?}", avg_bm25);
    println!("  📊 Rate: {:.1} searches/sec\n", 1.0 / avg_bm25.as_secs_f64());
    
    // Benchmark 4: Hierarchical Retrieval
    println!("🎯 Benchmark 4: Hierarchical Retrieval");
    let mut hierarchical_times = Vec::new();
    for _ in 0..20 {
        let start = Instant::now();
        let _results = manager.retrieve_hierarchical("programming language", workspace_id, 10)?;
        hierarchical_times.push(start.elapsed());
    }
    
    let avg_hierarchical = hierarchical_times.iter().sum::<std::time::Duration>() / hierarchical_times.len() as u32;
    println!("  ✅ Ran 20 hierarchical searches");
    println!("  ⏱️  Average: {:?}", avg_hierarchical);
    println!("  📊 Rate: {:.1} searches/sec\n", 1.0 / avg_hierarchical.as_secs_f64());
    
    // Benchmark 5: Consolidation
    println!("🔄 Benchmark 5: Consolidation");
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let start = Instant::now();
    let synopsis = manager.consolidate(date).await?;
    let consolidation_time = start.elapsed();
    
    println!("  ✅ Consolidated {} episodes", 100);
    println!("  ⏱️  Time: {:?}", consolidation_time);
    println!("  📊 Synopsis: {} insights\n", synopsis.key_insights.len());
    
    // Benchmark 6: Memory Stats
    println!("📊 Benchmark 6: Memory Stats");
    let start = Instant::now();
    let stats = manager.get_memory_stats(workspace_id)?;
    let stats_time = start.elapsed();
    
    println!("  ✅ Retrieved stats");
    println!("  ⏱️  Time: {:?}", stats_time);
    println!("  📈 Active episodes: {}", stats.active_episodes);
    println!("  📈 Knowledge count: {}\n", stats.knowledge_count);
    
    // Summary
    println!("═══════════════════════════════════════════");
    println!("📊 SUMMARY");
    println!("═══════════════════════════════════════════");
    println!("Episode Storage:        {:?} ({:.1}/sec)", avg_episode, 1.0 / avg_episode.as_secs_f64());
    println!("Semantic Storage:       {:?} ({:.1}/sec)", avg_semantic, 1.0 / avg_semantic.as_secs_f64());
    println!("BM25 Search:            {:?} ({:.1}/sec)", avg_bm25, 1.0 / avg_bm25.as_secs_f64());
    println!("Hierarchical Retrieval: {:?} ({:.1}/sec)", avg_hierarchical, 1.0 / avg_hierarchical.as_secs_f64());
    println!("Consolidation:          {:?}", consolidation_time);
    println!("Memory Stats:           {:?}", stats_time);
    println!("═══════════════════════════════════════════\n");
    
    // Assertions
    assert!(avg_episode.as_millis() < 100, "Episode storage should be < 100ms");
    assert!(avg_hierarchical.as_millis() < 10, "Hierarchical search should be < 10ms");
    assert!(consolidation_time.as_secs() < 10, "Consolidation should be < 10s");
    
    println!("✅ All benchmarks passed!\n");
    
    Ok(())
}
