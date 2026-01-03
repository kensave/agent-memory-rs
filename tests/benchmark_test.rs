use memory_rs::{FastEmbedder, ModelType};
use std::time::Instant;

#[test]
fn test_embed_single_text() {
    let embedder = FastEmbedder::new().unwrap();
    let result = embedder.embed("test text");
    
    assert!(result.is_ok());
    let embedding = result.unwrap();
    assert!(!embedding.is_empty());
    assert!(embedding.iter().any(|&x| x != 0.0));
}

#[test]
fn test_embedding_dimensions() {
    let miniml = FastEmbedder::with_model(ModelType::MiniLM).unwrap();
    let nomic = FastEmbedder::with_model(ModelType::Nomic).unwrap();
    
    let miniml_embed = miniml.embed("test").unwrap();
    let nomic_embed = nomic.embed("test").unwrap();
    
    assert_eq!(miniml_embed.len(), 384);
    assert_eq!(nomic_embed.len(), 768);
}

#[test]
fn benchmark_embedding_speed() {
    let models = vec![
        ("MiniLM", ModelType::MiniLM),
        ("Nomic", ModelType::Nomic),
    ];
    
    for (name, model_type) in models {
        let embedder = FastEmbedder::with_model(model_type).unwrap();
        let start = Instant::now();
        let _embedding = embedder.embed("benchmark text").unwrap();
        let duration = start.elapsed();
        
        println!("{}: {}μs", name, duration.as_micros());
        assert!(duration.as_millis() < 1000); // Should be fast
    }
}
