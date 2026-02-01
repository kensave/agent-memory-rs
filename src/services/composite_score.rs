use crate::models::dtos::CompositeScore;
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct CompositeScoreCalculator {
    recency_weight: f64,
    relevance_weight: f64,
    utility_weight: f64,
    decay_lambda: f64,
}

impl CompositeScoreCalculator {
    pub fn new() -> Self {
        Self {
            recency_weight: 0.3,
            relevance_weight: 0.4,
            utility_weight: 0.3,
            decay_lambda: 0.1,
        }
    }

    pub fn with_weights(recency: f64, relevance: f64, utility: f64) -> Self {
        Self {
            recency_weight: recency,
            relevance_weight: relevance,
            utility_weight: utility,
            decay_lambda: 0.1,
        }
    }

    pub fn calculate_recency(&self, timestamp: &str) -> Result<f64> {
        let created = parse_timestamp(timestamp)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let days_since = (now - created) as f64 / 86400.0;
        Ok((-self.decay_lambda * days_since).exp())
    }

    pub fn calculate_relevance(&self, similarity: f64) -> f64 {
        similarity.max(0.0).min(1.0)
    }

    pub fn calculate_utility(&self, access_count: i64, success_rate: f64, feedback: f64) -> f64 {
        let normalized_access = (access_count as f64 / 100.0).min(1.0);
        (normalized_access * 0.4 + success_rate * 0.4 + feedback * 0.2).max(0.0).min(1.0)
    }

    pub fn calculate_composite(&self, recency: f64, relevance: f64, utility: f64) -> CompositeScore {
        let combined = recency * self.recency_weight 
                     + relevance * self.relevance_weight 
                     + utility * self.utility_weight;
        
        CompositeScore {
            recency,
            relevance,
            utility,
            combined,
        }
    }

    pub fn calculate_for_memory(
        &self,
        timestamp: &str,
        similarity: f64,
        access_count: i64,
        success_rate: f64,
        feedback: f64,
    ) -> Result<CompositeScore> {
        let recency = self.calculate_recency(timestamp)?;
        let relevance = self.calculate_relevance(similarity);
        let utility = self.calculate_utility(access_count, success_rate, feedback);
        Ok(self.calculate_composite(recency, relevance, utility))
    }
}

impl Default for CompositeScoreCalculator {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_timestamp(timestamp: &str) -> Result<u64> {
    use chrono::{DateTime, NaiveDateTime};
    
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
        return Ok(dt.timestamp() as u64);
    }
    
    if let Ok(dt) = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt.and_utc().timestamp() as u64);
    }
    
    anyhow::bail!("Invalid timestamp format: {}", timestamp)
}
