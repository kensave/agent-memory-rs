use memory_rs::{FastEmbedder, ModelType};

#[tokio::test]
async fn test_real_model_loading() {
    let mut embedder = FastEmbedder::with_model(ModelType::MiniLM).unwrap();
    
    // This will download the model if not cached
    let result = embedder.load_model().await;
    
    if result.is_ok() {
        // Test embedding with real model
        let embedding = embedder.embed("Hello, world!").unwrap();
        assert_eq!(embedding.len(), 384);
        
        // Check that embedding is normalized (roughly)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.1, "Embedding should be normalized, got norm: {}", norm);
        
        println!("Real model embedding norm: {}", norm);
    } else {
        println!("Skipping real model test (download failed): {:?}", result.err());
    }
}

#[tokio::test]
async fn test_model_comparison_real() {
    let test_text = "This is a test sentence for embedding comparison.";
    
    // Test MiniLM
    let mut miniml = FastEmbedder::with_model(ModelType::MiniLM).unwrap();
    if miniml.load_model().await.is_ok() {
        let miniml_embed = miniml.embed(test_text).unwrap();
        println!("MiniLM real embedding dims: {}", miniml_embed.len());
        assert_eq!(miniml_embed.len(), 384);
    }
    
    // Note: Nomic model might be large, so we'll test it separately if needed
}
