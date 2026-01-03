use anyhow::Result;
use hf_hub::api::sync::Api;
use hf_hub::{Repo, RepoType};
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
        
        // Use the WORKING approach from semantic-search-client
        let api = hf_hub::api::sync::Api::new()?;
        let repo = api.repo(Repo::with_revision(
            repo_id.to_string(),
            RepoType::Model,
            "main".to_string(),
        ));
        
        // Download model file
        if !model_path.exists() {
            println!("  Downloading model.safetensors...");
            let model_file = repo.get("model.safetensors")?;
            std::fs::copy(model_file, &model_path)?;
            println!("  ✅ Model downloaded");
        }
        
        // Download tokenizer file  
        if !tokenizer_path.exists() {
            println!("  Downloading tokenizer.json...");
            let tokenizer_file = repo.get("tokenizer.json")?;
            std::fs::copy(tokenizer_file, &tokenizer_path)?;
            println!("  ✅ Tokenizer downloaded");
        }
        
        println!("✅ Model ready: {:?}", model_dir);
        Ok((model_path, tokenizer_path))
    }
}
