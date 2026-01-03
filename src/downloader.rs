use anyhow::Result;
use hf_hub::api::tokio::Api;
use std::path::PathBuf;
use tokio::fs;

pub struct ModelDownloader {
    cache_dir: PathBuf,
}

impl ModelDownloader {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("memory-rs");
        
        Ok(Self { cache_dir })
    }
    
    pub async fn download_model(&self, repo_id: &str) -> Result<(PathBuf, PathBuf)> {
        let model_dir = self.cache_dir.join(repo_id.replace('/', "_"));
        
        // Create cache directory
        fs::create_dir_all(&model_dir).await?;
        
        let model_path = model_dir.join("model.safetensors");
        let tokenizer_path = model_dir.join("tokenizer.json");
        
        // Check if already downloaded
        if model_path.exists() && tokenizer_path.exists() {
            println!("✅ Using cached model: {}", repo_id);
            return Ok((model_path, tokenizer_path));
        }
        
        println!("📥 Downloading model: {}", repo_id);
        
        // Create API instance with proper configuration
        let api = Api::new().map_err(|e| anyhow::anyhow!("Failed to create API: {}", e))?;
        let repo = api.model(repo_id.to_string());
        
        // Download model file
        if !model_path.exists() {
            println!("  Downloading model.safetensors...");
            match repo.get("model.safetensors").await {
                Ok(model_file) => {
                    fs::copy(&model_file, &model_path).await?;
                    println!("  ✅ Model downloaded");
                }
                Err(e) => {
                    println!("  ⚠️  model.safetensors not found, trying pytorch_model.bin");
                    let model_file = repo.get("pytorch_model.bin").await
                        .map_err(|e| anyhow::anyhow!("Failed to download model files: {}", e))?;
                    fs::copy(&model_file, &model_path).await?;
                    println!("  ✅ Model downloaded (pytorch_model.bin)");
                }
            }
        }
        
        // Download tokenizer file  
        if !tokenizer_path.exists() {
            println!("  Downloading tokenizer.json...");
            let tokenizer_file = repo.get("tokenizer.json").await
                .map_err(|e| anyhow::anyhow!("Failed to download tokenizer: {}", e))?;
            fs::copy(&tokenizer_file, &tokenizer_path).await?;
            println!("  ✅ Tokenizer downloaded");
        }
        
        println!("✅ Model ready: {:?}", model_dir);
        Ok((model_path, tokenizer_path))
    }
}
