use anyhow::Result;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig, DTYPE};
use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer};
use crate::models::ModelType;
use crate::downloader::ModelDownloader;

pub struct FastEmbedder {
    model: Option<BertModel>,
    tokenizer: Option<Tokenizer>,
    device: Device,
    model_type: ModelType,
}

impl FastEmbedder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            model: None,
            tokenizer: None,
            device: get_best_device(),
            model_type: ModelType::MiniLM,
        })
    }
    
    pub fn with_model(model_type: ModelType) -> Result<Self> {
        Ok(Self {
            model: None,
            tokenizer: None,
            device: get_best_device(),
            model_type,
        })
    }
    
    pub async fn load_model(&mut self) -> Result<()> {
        let downloader = ModelDownloader::new()?;
        let (model_path, tokenizer_path) = downloader.download_model(self.model_type.repo_id()).await?;
        
        // Load and configure tokenizer with padding (done once, not per-batch)
        let mut tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;
        tokenizer.with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            ..Default::default()
        }));
        self.tokenizer = Some(tokenizer);
        
        // Load model
        let config = self.get_bert_config();
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[&model_path], DTYPE, &self.device)? };
        self.model = Some(BertModel::load(vb, &config)?);
        
        println!("Loaded model: {:?} on {:?}", self.model_type, self.device);
        Ok(())
    }
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match (&self.model, &self.tokenizer) {
            (Some(model), Some(tokenizer)) => {
                self.embed_batch_internal(&[text], model, tokenizer)?
                    .into_iter().next().ok_or_else(|| anyhow::anyhow!("No embedding"))
            }
            _ => Ok(self.generate_mock_embedding(self.model_type.dimensions())),
        }
    }
    
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        match (&self.model, &self.tokenizer) {
            (Some(model), Some(tokenizer)) => self.embed_batch_internal(texts, model, tokenizer),
            _ => Ok(texts.iter().map(|_| self.generate_mock_embedding(self.model_type.dimensions())).collect()),
        }
    }
    
    fn embed_batch_internal(&self, texts: &[&str], model: &BertModel, tokenizer: &Tokenizer) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        
        let encodings = tokenizer.encode_batch(texts.to_vec(), true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
        
        // Build tensors directly (no create-then-stack)
        let (token_ids, attention_mask) = self.create_batch_tensors_direct(&encodings)?;
        let token_type_ids = token_ids.zeros_like()?;
        
        // Single forward pass
        let embeddings = model.forward(&token_ids, &token_type_ids, Some(&attention_mask))?;
        
        // Mean pooling + normalize
        let pooled = embeddings.mean(1)?;
        let normalized = normalize_l2(&pooled)?;
        
        Ok(normalized.to_vec2()?)
    }
    
    fn create_batch_tensors_direct(&self, encodings: &[tokenizers::Encoding]) -> Result<(Tensor, Tensor)> {
        let batch_size = encodings.len();
        let max_len = encodings.iter().map(|e| e.len()).max().unwrap_or(0);
        
        let mut token_ids = vec![0u32; batch_size * max_len];
        let mut attention_mask = vec![0u32; batch_size * max_len];
        
        for (i, encoding) in encodings.iter().enumerate() {
            let offset = i * max_len;
            token_ids[offset..offset + encoding.get_ids().len()].copy_from_slice(encoding.get_ids());
            attention_mask[offset..offset + encoding.get_attention_mask().len()].copy_from_slice(encoding.get_attention_mask());
        }
        
        Ok((
            Tensor::from_vec(token_ids, (batch_size, max_len), &self.device)?,
            Tensor::from_vec(attention_mask, (batch_size, max_len), &self.device)?,
        ))
    }
    
    fn get_bert_config(&self) -> BertConfig {
        match self.model_type {
            ModelType::MiniLM => BertConfig {
                vocab_size: 30522, hidden_size: 384, num_hidden_layers: 6, num_attention_heads: 12,
                intermediate_size: 1536, hidden_act: candle_transformers::models::bert::HiddenAct::Gelu,
                hidden_dropout_prob: 0.0, max_position_embeddings: 512, type_vocab_size: 2,
                initializer_range: 0.02, layer_norm_eps: 1e-12, pad_token_id: 0,
                position_embedding_type: candle_transformers::models::bert::PositionEmbeddingType::Absolute,
                use_cache: false, classifier_dropout: None, model_type: Some("bert".to_string()),
            },
            ModelType::Nomic => BertConfig {
                vocab_size: 30528, hidden_size: 768, num_hidden_layers: 12, num_attention_heads: 12,
                intermediate_size: 3072, hidden_act: candle_transformers::models::bert::HiddenAct::Gelu,
                hidden_dropout_prob: 0.0, max_position_embeddings: 8192, type_vocab_size: 2,
                initializer_range: 0.02, layer_norm_eps: 1e-12, pad_token_id: 0,
                position_embedding_type: candle_transformers::models::bert::PositionEmbeddingType::Absolute,
                use_cache: false, classifier_dropout: None, model_type: Some("bert".to_string()),
            },
            ModelType::BgeSmall => BertConfig {
                vocab_size: 30522, hidden_size: 384, num_hidden_layers: 12, num_attention_heads: 12,
                intermediate_size: 1536, hidden_act: candle_transformers::models::bert::HiddenAct::Gelu,
                hidden_dropout_prob: 0.1, max_position_embeddings: 512, type_vocab_size: 2,
                initializer_range: 0.02, layer_norm_eps: 1e-12, pad_token_id: 0,
                position_embedding_type: candle_transformers::models::bert::PositionEmbeddingType::Absolute,
                use_cache: false, classifier_dropout: None, model_type: Some("bert".to_string()),
            },
        }
    }
    
    fn generate_mock_embedding(&self, dims: usize) -> Vec<f32> {
        let mut embedding: Vec<f32> = (0..dims).map(|i| ((i as f32 * 0.1) % 1.0) * 2.0 - 1.0).collect();
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 { embedding.iter_mut().for_each(|v| *v /= norm); }
        embedding
    }
}

fn get_best_device() -> Device {
    #[cfg(feature = "metal")]
    if let Ok(device) = Device::new_metal(0) { return device; }
    #[cfg(feature = "cuda")]
    if let Ok(device) = Device::new_cuda(0) { return device; }
    Device::Cpu
}

fn normalize_l2(tensor: &Tensor) -> Result<Tensor> {
    Ok(tensor.broadcast_div(&tensor.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}
