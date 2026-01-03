use anyhow::Result;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig, DTYPE};
use tokenizers::Tokenizer;
use crate::models::{ModelType, QuantizationType};
use crate::downloader::ModelDownloader;

pub struct FastEmbedder {
    model: Option<BertModel>,
    tokenizer: Option<Tokenizer>,
    device: Device,
    model_type: ModelType,
    quantization: QuantizationType,
}

impl FastEmbedder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            model: None,
            tokenizer: None,
            device: Device::Cpu,
            model_type: ModelType::MiniLM,
            quantization: QuantizationType::None,
        })
    }
    
    pub fn with_model(model_type: ModelType) -> Result<Self> {
        Ok(Self {
            model: None,
            tokenizer: None,
            device: Device::Cpu,
            model_type,
            quantization: QuantizationType::None,
        })
    }
    
    pub fn with_quantization(quantization: QuantizationType) -> Result<Self> {
        Ok(Self {
            model: None,
            tokenizer: None,
            device: Device::Cpu,
            model_type: ModelType::MiniLM,
            quantization,
        })
    }
    
    pub async fn load_model(&mut self) -> Result<()> {
        let downloader = ModelDownloader::new()?;
        let (model_path, tokenizer_path) = downloader.download_model(self.model_type.repo_id()).await?;
        
        // Load tokenizer
        self.tokenizer = Some(Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?);
        
        // Load model
        let config = self.get_bert_config();
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[&model_path], DTYPE, &self.device)? };
        self.model = Some(BertModel::load(vb, &config)?);
        
        println!("Loaded model: {:?}", self.model_type);
        Ok(())
    }
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match (&self.model, &self.tokenizer) {
            (Some(model), Some(tokenizer)) => {
                self.embed_with_model(text, model, tokenizer)
            }
            _ => {
                // Fallback to mock implementation if model not loaded
                Ok(self.generate_mock_embedding(self.model_type.dimensions()))
            }
        }
    }
    
    fn embed_with_model(&self, text: &str, model: &BertModel, tokenizer: &Tokenizer) -> Result<Vec<f32>> {
        // Tokenize input
        let encoding = tokenizer.encode(text, false)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
        let tokens = encoding.get_ids();
        
        // Create tensors
        let token_ids = Tensor::new(tokens, &self.device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;
        
        // Forward pass
        let embeddings = model.forward(&token_ids, &token_type_ids, None)?;
        
        // Mean pooling
        let pooled = embeddings.mean(1)?.squeeze(0)?;
        
        // Normalize
        let norm = pooled.sqr()?.sum_all()?.sqrt()?;
        let normalized = pooled.broadcast_div(&norm)?;
        
        // Convert to Vec<f32>
        Ok(normalized.to_vec1()?)
    }
    
    fn get_bert_config(&self) -> BertConfig {
        match self.model_type {
            ModelType::MiniLM => BertConfig {
                vocab_size: 30522,
                hidden_size: 384,
                num_hidden_layers: 6,
                num_attention_heads: 12,
                intermediate_size: 1536,
                hidden_act: candle_transformers::models::bert::HiddenAct::Gelu,
                hidden_dropout_prob: 0.0,
                max_position_embeddings: 512,
                type_vocab_size: 2,
                initializer_range: 0.02,
                layer_norm_eps: 1e-12,
                pad_token_id: 0,
                position_embedding_type: candle_transformers::models::bert::PositionEmbeddingType::Absolute,
                use_cache: false,
                classifier_dropout: None,
                model_type: Some("bert".to_string()),
            },
            ModelType::Nomic => BertConfig {
                vocab_size: 30522,
                hidden_size: 768,
                num_hidden_layers: 12,
                num_attention_heads: 12,
                intermediate_size: 3072,
                hidden_act: candle_transformers::models::bert::HiddenAct::Gelu,
                hidden_dropout_prob: 0.0,
                max_position_embeddings: 8192,
                type_vocab_size: 2,
                initializer_range: 0.02,
                layer_norm_eps: 1e-12,
                pad_token_id: 0,
                position_embedding_type: candle_transformers::models::bert::PositionEmbeddingType::Absolute,
                use_cache: false,
                classifier_dropout: None,
                model_type: Some("bert".to_string()),
            },
        }
    }
    
    fn generate_mock_embedding(&self, dims: usize) -> Vec<f32> {
        // Generate normalized random-like embeddings for testing
        let mut embedding = vec![0.0; dims];
        for (i, val) in embedding.iter_mut().enumerate() {
            *val = ((i as f32 * 0.1) % 1.0) * 2.0 - 1.0; // Values between -1 and 1
        }
        
        // Normalize the embedding
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        embedding
    }
}
