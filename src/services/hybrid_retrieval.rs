use crate::storage::database::Database;
use crate::embedder::FastEmbedder;
use anyhow::Result;
use rusqlite::params;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct HybridRetrievalEngine {
    db: Database,
    embedder: Option<Arc<Mutex<FastEmbedder>>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HybridSearchResult {
    pub memory_type: String,
    pub content: String,
    pub score: f64,
    pub id: i64,
}

impl HybridRetrievalEngine {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            embedder: None,
        }
    }
    
    pub fn with_embedder(db: Database, embedder: Arc<Mutex<FastEmbedder>>) -> Self {
        Self {
            db,
            embedder: Some(embedder),
        }
    }

    pub fn search_bm25(&self, query: &str, workspace_id: i64, limit: usize) -> Result<Vec<HybridSearchResult>> {
        let keywords: Vec<&str> = query.split_whitespace().collect();
        let mut results = Vec::new();

        // Search memories
        let memories = self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, text, importance_score, created_at
                 FROM memories
                 WHERE workspace_id = ?
                 ORDER BY created_at DESC
                 LIMIT 100"
            )?;
            
            let rows = stmt.query_map(params![workspace_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?;
            
            Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
        })?;

        for (id, text, importance, _created_at) in memories {
            let score = self.calculate_bm25_score(&text, &keywords) * importance;
            if score > 0.0 {
                results.push(HybridSearchResult {
                    memory_type: "semantic".to_string(),
                    content: text,
                    score,
                    id,
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        Ok(results)
    }

    fn calculate_bm25_score(&self, text: &str, keywords: &[&str]) -> f64 {
        let text_lower = text.to_lowercase();
        let mut score = 0.0;
        
        for keyword in keywords {
            let keyword_lower = keyword.to_lowercase();
            let count = text_lower.matches(&keyword_lower).count() as f64;
            if count > 0.0 {
                score += (count + 1.0).ln();
            }
        }
        
        score
    }

    pub fn hybrid_search(&self, query: &str, workspace_id: i64, limit: usize) -> Result<Vec<HybridSearchResult>> {
        // Get BM25 results
        let bm25_results = self.search_bm25(query, workspace_id, limit * 2)?;
        
        // Get vector results if embedder available
        let vector_results = if let Some(ref embedder) = self.embedder {
            let query_emb = embedder.lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire embedder lock"))?
                .embed(query)?;
            
            let embedding_blob: Vec<u8> = query_emb.iter()
                .flat_map(|&f| f.to_le_bytes())
                .collect();
            
            self.db.execute(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT m.id, m.text, vec_distance_cosine(v.embedding, ?1) AS distance
                     FROM memories m
                     JOIN vec0 v ON v.memory_id = m.id
                     WHERE m.workspace_id = ?
                     ORDER BY distance ASC
                     LIMIT ?"
                )?;
                
                let rows = stmt.query_map(rusqlite::params![embedding_blob, workspace_id, limit * 2], |row| {
                    let id: i64 = row.get(0)?;
                    let text: String = row.get(1)?;
                    let distance: f64 = row.get(2)?;
                    
                    Ok(HybridSearchResult {
                        memory_type: "semantic".to_string(),
                        content: text,
                        score: 1.0 - distance,
                        id,
                    })
                })?;
                
                Ok(rows.filter_map(Result::ok).collect::<Vec<_>>())
            })?
        } else {
            Vec::new()
        };
        
        // RRF fusion: combine BM25 and vector rankings
        let mut fused_scores: HashMap<i64, f64> = HashMap::new();
        
        // Add BM25 scores
        for (rank, result) in bm25_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f64);
            *fused_scores.entry(result.id).or_insert(0.0) += rrf_score;
        }
        
        // Add vector scores
        for (rank, result) in vector_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f64);
            *fused_scores.entry(result.id).or_insert(0.0) += rrf_score;
        }
        
        // Combine all unique results
        let mut all_results: HashMap<i64, HybridSearchResult> = HashMap::new();
        for result in bm25_results.into_iter().chain(vector_results.into_iter()) {
            all_results.entry(result.id).or_insert(result);
        }
        
        // Apply fused scores
        let mut final_results: Vec<HybridSearchResult> = all_results.into_iter()
            .map(|(id, mut r)| {
                r.score = *fused_scores.get(&id).unwrap_or(&0.0);
                r
            })
            .collect();

        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        final_results.truncate(limit);
        Ok(final_results)
    }

    pub fn search_by_type(&self, query: &str, workspace_id: i64, memory_type: &str, limit: usize) -> Result<Vec<HybridSearchResult>> {
        match memory_type {
            "semantic" => self.search_bm25(query, workspace_id, limit),
            "episodic" => self.search_episodes(query, workspace_id, limit),
            "procedural" => self.search_procedures(query, workspace_id, limit),
            _ => self.hybrid_search(query, workspace_id, limit),
        }
    }

    fn search_episodes(&self, query: &str, workspace_id: i64, limit: usize) -> Result<Vec<HybridSearchResult>> {
        // Vector search if embedder available
        if let Some(ref embedder) = self.embedder {
            let query_emb = embedder.lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire embedder lock"))?
                .embed(query)?;
            let embedding_blob: Vec<u8> = query_emb.iter()
                .flat_map(|&f| f.to_le_bytes())
                .collect();
            
            let vector_results: Result<Vec<HybridSearchResult>> = self.db.execute(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT e.id, e.event_type, e.outcome, e.context,
                            vec_distance_cosine(v.embedding, ?1) AS distance
                     FROM episodes e
                     JOIN vec0 v ON v.memory_id = -e.id
                     WHERE e.workspace_id = ? AND e.archived = 0
                     ORDER BY distance ASC
                     LIMIT ?"
                )?;
                
                let rows = stmt.query_map(params![embedding_blob, workspace_id, limit], |row| {
                    let event_type: String = row.get(1)?;
                    let outcome: Option<String> = row.get(2)?;
                    let context: String = row.get(3)?;
                    let distance: f64 = row.get(4)?;
                    
                    Ok(HybridSearchResult {
                        memory_type: "episodic".to_string(),
                        content: format!("{} {} {}", event_type, outcome.unwrap_or_default(), context),
                        score: 1.0 - distance,
                        id: row.get(0)?,
                    })
                })?;
                
                Ok(rows.filter_map(Result::ok).collect())
            });
            
            if let Ok(results) = vector_results {
                if !results.is_empty() {
                    return Ok(results);
                }
            }
        }
        
        // Fallback to BM25
        let keywords: Vec<&str> = query.split_whitespace().collect();
        
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, event_type, outcome, context, timestamp
                 FROM episodes
                 WHERE workspace_id = ? AND archived = 0
                 ORDER BY timestamp DESC
                 LIMIT 100"
            )?;
            
            let rows = stmt.query_map(params![workspace_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })?;
            
            let mut results = Vec::new();
            for row in rows.filter_map(Result::ok) {
                let (id, event_type, outcome, context, _timestamp) = row;
                let text = format!("{} {} {}", event_type, outcome.unwrap_or_default(), context);
                let score = self.calculate_bm25_score(&text, &keywords);
                
                if score > 0.0 {
                    results.push(HybridSearchResult {
                        memory_type: "episodic".to_string(),
                        content: text,
                        score,
                        id,
                    });
                }
            }
            
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            results.truncate(limit);
            Ok(results)
        })
    }

    fn search_procedures(&self, query: &str, workspace_id: i64, limit: usize) -> Result<Vec<HybridSearchResult>> {
        let keywords: Vec<&str> = query.split_whitespace().collect();
        
        self.db.execute(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, success_rate
                 FROM procedures
                 WHERE workspace_id = ?
                 ORDER BY usage_count DESC
                 LIMIT 100"
            )?;
            
            let rows = stmt.query_map(params![workspace_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                ))
            })?;
            
            let mut results = Vec::new();
            for row in rows.filter_map(Result::ok) {
                let (id, name, success_rate) = row;
                let score = self.calculate_bm25_score(&name, &keywords) * success_rate;
                
                if score > 0.0 {
                    results.push(HybridSearchResult {
                        memory_type: "procedural".to_string(),
                        content: name,
                        score,
                        id,
                    });
                }
            }
            
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            results.truncate(limit);
            Ok(results)
        })
    }
}
