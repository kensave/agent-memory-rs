use memory_rs::{FastEmbedder, QuantizationType};
use std::time::Instant;

#[test]
fn test_quantization_types() {
    let quantizations = vec![
        ("Full Precision", QuantizationType::None),
        ("Int8 Quantized", QuantizationType::Int8),
    ];
    
    for (name, quant_type) in quantizations {
        let embedder = FastEmbedder::with_quantization(quant_type).unwrap();
        let embedding = embedder.embed("quantization test").unwrap();
        
        println!("{}: dims = {}", name, embedding.len());
        assert!(!embedding.is_empty());
        assert!(embedding.iter().any(|&x| x != 0.0));
    }
}

#[test]
fn benchmark_quantization_performance() {
    let text = "Performance test for quantized models with longer text content";
    
    // Full precision
    let full_embedder = FastEmbedder::with_quantization(QuantizationType::None).unwrap();
    let start = Instant::now();
    let full_embed = full_embedder.embed(text).unwrap();
    let full_time = start.elapsed();
    
    // Quantized
    let quant_embedder = FastEmbedder::with_quantization(QuantizationType::Int8).unwrap();
    let start = Instant::now();
    let quant_embed = quant_embedder.embed(text).unwrap();
    let quant_time = start.elapsed();
    
    println!("Full precision: {}μs", full_time.as_micros());
    println!("Int8 quantized: {}μs", quant_time.as_micros());
    
    // Both should produce embeddings
    assert_eq!(full_embed.len(), quant_embed.len());
    
    // Quantized might be faster (or at least not much slower)
    // This is a placeholder - real implementation will show actual differences
}

#[test]
fn test_memory_usage_simulation() {
    // Simulate memory usage differences
    let full_embedder = FastEmbedder::with_quantization(QuantizationType::None).unwrap();
    let quant_embedder = FastEmbedder::with_quantization(QuantizationType::Int8).unwrap();
    
    // In real implementation, quantized should use ~4x less memory
    let full_embed = full_embedder.embed("memory test").unwrap();
    let quant_embed = quant_embedder.embed("memory test").unwrap();
    
    // For now, just ensure both work
    assert_eq!(full_embed.len(), quant_embed.len());
    
    println!("Full precision memory: {} bytes (simulated)", full_embed.len() * 4);
    println!("Int8 quantized memory: {} bytes (simulated)", quant_embed.len() * 1);
}
