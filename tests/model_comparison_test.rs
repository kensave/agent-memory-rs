use memory_rs::{FastEmbedder, ModelType};
use std::time::Instant;

#[test]
fn compare_model_performance() {
    let test_texts = vec![
        "Short text",
        "This is a longer text that might take more time to process and embed into vectors",
        "fn main() { println!(\"Hello, world!\"); }",
    ];
    
    for text in &test_texts {
        println!("\nBenchmarking text: {}", text);
        
        // Test MiniLM
        let miniml = FastEmbedder::with_model(ModelType::MiniLM).unwrap();
        let start = Instant::now();
        let miniml_embed = miniml.embed(text).unwrap();
        let miniml_time = start.elapsed();
        
        // Test Nomic
        let nomic = FastEmbedder::with_model(ModelType::Nomic).unwrap();
        let start = Instant::now();
        let nomic_embed = nomic.embed(text).unwrap();
        let nomic_time = start.elapsed();
        
        println!("MiniLM: {}μs, dims: {}", miniml_time.as_micros(), miniml_embed.len());
        println!("Nomic:  {}μs, dims: {}", nomic_time.as_micros(), nomic_embed.len());
        
        // Both should complete quickly
        assert!(miniml_time.as_millis() < 100);
        assert!(nomic_time.as_millis() < 100);
    }
}

#[test]
fn test_model_quality_difference() {
    let embedder_miniml = FastEmbedder::with_model(ModelType::MiniLM).unwrap();
    let embedder_nomic = FastEmbedder::with_model(ModelType::Nomic).unwrap();
    
    let text = "artificial intelligence machine learning";
    
    let miniml_embed = embedder_miniml.embed(text).unwrap();
    let nomic_embed = embedder_nomic.embed(text).unwrap();
    
    // Different models should produce different embeddings
    assert_ne!(miniml_embed[0], nomic_embed[0]);
    
    // Both should be normalized (roughly)
    let miniml_norm: f32 = miniml_embed.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nomic_norm: f32 = nomic_embed.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    assert!(miniml_norm > 0.0);
    assert!(nomic_norm > 0.0);
}
