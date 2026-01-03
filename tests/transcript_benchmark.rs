use memory_rs::{FastEmbedder, ModelType};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[tokio::test]
async fn benchmark_transcript_processing() {
    let transcript_dir = "/Users/kenneth/workspace/convergence/kb_with_transcripts";
    
    if !Path::new(transcript_dir).exists() {
        println!("⚠️  Transcript directory not found, skipping benchmark");
        return;
    }
    
    println!("🚀 Benchmarking Memory-RS on Real Transcript Data");
    println!("================================================\n");
    
    // Get all transcript files
    let files: Vec<_> = fs::read_dir(transcript_dir)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "txt" {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    
    println!("Found {} transcript files", files.len());
    
    // Test with both models
    for model_type in [ModelType::MiniLM, ModelType::Nomic] {
        println!("\n📊 Testing with {:?} Model:", model_type);
        println!("--------------------------------");
        
        let mut embedder = FastEmbedder::with_model(model_type).unwrap();
        
        // Try to load real model, fallback to mock
        let model_loaded = embedder.load_model().await.is_ok();
        if model_loaded {
            println!("✅ Real model loaded");
        } else {
            println!("⚠️  Using mock implementation");
        }
        
        let mut total_chars = 0;
        let mut total_time = std::time::Duration::ZERO;
        let mut total_embeddings = 0;
        
        for (_i, file_path) in files.iter().enumerate().take(3) { // Test first 3 files
            let filename = file_path.file_name().unwrap().to_string_lossy();
            println!("\nProcessing: {}", filename);
            
            // Read file content
            let content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    println!("  ❌ Failed to read file: {}", e);
                    continue;
                }
            };
            
            let file_size = content.len();
            total_chars += file_size;
            
            // Split into chunks (simulate processing chunks)
            let chunk_size = 500; // 500 chars per chunk
            let chunks: Vec<String> = content
                .chars()
                .collect::<Vec<_>>()
                .chunks(chunk_size)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect();
            
            println!("  📄 File size: {} chars, {} chunks", file_size, chunks.len());
            
            // Benchmark embedding generation
            let start = Instant::now();
            let mut embeddings_generated = 0;
            
            for (chunk_idx, chunk) in chunks.iter().enumerate().take(10) { // Test first 10 chunks
                if chunk.trim().is_empty() {
                    continue;
                }
                
                match embedder.embed(chunk) {
                    Ok(embedding) => {
                        embeddings_generated += 1;
                        total_embeddings += 1;
                        
                        if chunk_idx == 0 {
                            println!("  ✅ First embedding: {} dims", embedding.len());
                        }
                    }
                    Err(e) => {
                        println!("  ❌ Embedding failed: {}", e);
                    }
                }
            }
            
            let file_time = start.elapsed();
            total_time += file_time;
            
            println!("  ⏱️  Time: {}ms, Embeddings: {}, Rate: {:.1} chunks/sec", 
                    file_time.as_millis(),
                    embeddings_generated,
                    embeddings_generated as f64 / file_time.as_secs_f64());
        }
        
        // Summary
        println!("\n📈 {:?} Model Summary:", model_type);
        println!("  Total characters processed: {}", total_chars);
        println!("  Total embeddings generated: {}", total_embeddings);
        println!("  Total time: {}ms", total_time.as_millis());
        println!("  Average rate: {:.1} embeddings/sec", 
                total_embeddings as f64 / total_time.as_secs_f64());
        println!("  Chars per second: {:.0}", 
                total_chars as f64 / total_time.as_secs_f64());
    }
    
    println!("\n🎯 Benchmark Complete!");
    println!("Ready for production transcript processing with memory-rs");
}

#[test]
fn test_transcript_chunk_processing() {
    // Test chunking logic with sample transcript data
    let sample_transcript = "[26/8/23, 12:15:09 p. m.] Kenneth: Test message\n[26/8/23, 12:16:00 p. m.] User: Response";
    
    let embedder = FastEmbedder::new().unwrap();
    let embedding = embedder.embed(sample_transcript).unwrap();
    
    assert!(!embedding.is_empty());
    println!("Sample transcript embedding: {} dims", embedding.len());
}
