#[derive(Clone, Copy, Debug)]
pub enum ModelType {
    MiniLM,
    Nomic,
    BgeSmall,
}

impl ModelType {
    pub fn repo_id(&self) -> &'static str {
        match self {
            Self::MiniLM => "sentence-transformers/all-MiniLM-L6-v2",
            Self::Nomic => "nomic-ai/nomic-embed-text-v1",
            Self::BgeSmall => "BAAI/bge-small-en-v1.5",
        }
    }
    
    pub fn dimensions(&self) -> usize {
        match self {
            Self::MiniLM => 384,
            Self::Nomic => 768,
            Self::BgeSmall => 384,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum QuantizationType {
    None,
    Int8,
}
