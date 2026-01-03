use memory_rs::{FastEmbedder, ModelType};

#[tokio::test]
async fn test_model_download_debug() {
    let mut embedder = FastEmbedder::with_model(ModelType::MiniLM).unwrap();
    
    println!("🔍 Testing model download...");
    match embedder.load_model().await {
        Ok(()) => {
            println!("✅ Model loaded successfully!");
            
            // Test embedding
            let embedding = embedder.embed("Hello world").unwrap();
            println!("✅ Real embedding generated: {} dims", embedding.len());
            
            // Check normalization
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            println!("📊 Embedding norm: {:.4}", norm);
        }
        Err(e) => {
            println!("❌ Model loading failed: {}", e);
            println!("🔍 Error details: {:?}", e);
        }
    }
}
