use anyhow::Result;
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::path::PathBuf;

pub struct ModelDownloader {
    api: Api,
}

impl ModelDownloader {
    pub fn new() -> Result<Self> {
        let api = Api::new()?;
        Ok(Self { api })
    }
    
    pub fn download_model(&self, repo_id: &str) -> Result<(PathBuf, PathBuf)> {
        let repo = self.api.repo(Repo::with_revision(
            repo_id.to_string(),
            RepoType::Model,
            "main".to_string(),
        ));
        
        // Check if already cached
        let model_path = repo.get("model.safetensors");
        let tokenizer_path = repo.get("tokenizer.json");
        
        match (model_path, tokenizer_path) {
            (Ok(m), Ok(t)) if m.exists() && t.exists() => {
                tracing::info!("Using cached model: {}", repo_id);
                Ok((m, t))
            }
            _ => {
                tracing::info!("Downloading model: {}", repo_id);
                
                // Download model file
                tracing::info!("Downloading model.safetensors...");
                let model_file = repo.get("model.safetensors")?;
                tracing::info!("Model downloaded");
                
                // Download tokenizer file  
                tracing::info!("Downloading tokenizer.json...");
                let tokenizer_file = repo.get("tokenizer.json")?;
                tracing::info!("Tokenizer downloaded");
                
                tracing::info!("Model ready: {:?}", model_file.parent().unwrap());
                Ok((model_file, tokenizer_file))
            }
        }
    }
}
