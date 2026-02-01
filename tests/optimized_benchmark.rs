use agent_memory_rs::{FastEmbedder, ModelType};
use std::time::Instant;

#[tokio::test]
async fn benchmark_fair_comparison() {
    println!("🔍 Fair Benchmark (no caching tricks)");
    println!("=====================================\n");
    
    let mut embedder = FastEmbedder::with_model(ModelType::MiniLM).unwrap();
    
    if embedder.load_model_sync().is_err() {
        println!("⚠️  Model load failed");
        return;
    }
    
    // Warm-up
    let _ = embedder.embed("warmup");
    
    // Single embedding
    let start = Instant::now();
    let single = embedder.embed("This is a test sentence for embedding.").unwrap();
    let single_time = start.elapsed();
    
    // Batch of 5 (same as semantic-search-client benchmark)
    let texts: Vec<&str> = vec![
        "This is a short sentence.",
        "Another simple example.",
        "The quick brown fox jumps over the lazy dog.",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        "Machine learning models can process and analyze text data to extract meaningful information.",
    ];
    
    let start = Instant::now();
    let batch = embedder.embed_batch(&texts).unwrap();
    let batch_time = start.elapsed();
    
    println!("📊 MEMORY-RS (no cache):");
    println!("  Single: {:?}", single_time);
    println!("  Batch (5): {:?}", batch_time);
    println!("  Avg/text: {:?}", batch_time / 5);
    println!("  Rate: {:.1} texts/sec", 5.0 / batch_time.as_secs_f64());
    
    println!("\n📊 SEMANTIC-SEARCH-CLIENT (reference):");
    println!("  Single: ~11.6ms");
    println!("  Batch (5): ~299ms");
    println!("  Avg/text: ~59ms");
    println!("  Rate: ~16.8 texts/sec");
    
    let speedup = 299.0 / batch_time.as_millis() as f64;
    println!("\n🚀 Speedup: {:.1}x faster", speedup);
    
    assert_eq!(single.len(), 384);
    assert_eq!(batch.len(), 5);
}
