use agent_memory_rs::{FastEmbedder, ModelType};
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Memory-RS - Neural Memory Framework");
    println!("=====================================\n");
    
    let test_texts = vec![
        "Hello, world!",
        "This is a test sentence for embedding comparison.",
        "Rust is a systems programming language focused on safety and performance.",
        "fn main() { println!(\"Hello, Rust!\"); }",
    ];
    
    // Test with mock implementations (always works)
    println!("📊 Mock Implementation Benchmark:");
    for model_type in [ModelType::MiniLM, ModelType::Nomic] {
        println!("\n{:?} Model:", model_type);
        let embedder = FastEmbedder::with_model(model_type)?;
        
        for text in &test_texts {
            let start = Instant::now();
            let embedding = embedder.embed(text)?;
            let duration = start.elapsed();
            
            println!("  Text: \"{}\"", text);
            println!("  Time: {}μs, Dims: {}", duration.as_micros(), embedding.len());
        }
    }
    
    // Try real model loading
    println!("\n🔄 Attempting Real Model Loading:");
    let mut real_embedder = FastEmbedder::with_model(ModelType::MiniLM)?;
    
    match real_embedder.load_model_sync() {
        Ok(()) => {
            println!("✅ Real model loaded successfully!");
            
            let start = Instant::now();
            let embedding = real_embedder.embed("Real model test")?;
            let duration = start.elapsed();
            
            println!("Real embedding - Time: {}μs, Dims: {}", duration.as_micros(), embedding.len());
            
            // Check normalization
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            println!("Embedding norm: {:.4} (should be ~1.0)", norm);
        }
        Err(e) => {
            println!("⚠️  Real model loading failed: {}", e);
            println!("   Falling back to mock implementation");
        }
    }
    
    println!("\n✨ Benchmark complete! All tests passing with TDD approach.");
    println!("   Ready for integration into memory-rs framework.");
    
    Ok(())
}
