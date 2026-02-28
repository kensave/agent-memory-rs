use agent_memory_rs::{WorkspaceManager, ModelType, services::hybrid_retrieval::HybridRetrievalEngine};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn extract_dia_id(text: &str) -> Option<String> {
    if let Some(start) = text.find("[D") {
        if let Some(end) = text[start..].find(']') {
            return Some(text[start+1..start+end].to_string());
        }
    }
    None
}

fn calculate_recall_at_k(retrieved_ids: &[String], evidence: &[String], k: usize) -> f64 {
    if evidence.is_empty() {
        return 0.0;
    }
    let top_k: Vec<_> = retrieved_ids.iter().take(k).collect();
    let found = evidence.iter().any(|e| top_k.contains(&e));
    if found { 1.0 } else { 0.0 }
}

fn calculate_mrr(retrieved_ids: &[String], evidence: &[String]) -> f64 {
    if evidence.is_empty() {
        return 0.0;
    }
    for (rank, id) in retrieved_ids.iter().enumerate() {
        if evidence.contains(id) {
            return 1.0 / (rank + 1) as f64;
        }
    }
    0.0
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <conversation_index>", args[0]);
        std::process::exit(1);
    }
    
    let conv_idx: usize = args[1].parse()?;
    
    let data_path = std::env::var("LOCOMO_DATA_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap()).join("LoCoMo/data/locomo10.json")
        });
    
    let data = fs::read_to_string(&data_path)?;
    let conversations: Vec<Value> = serde_json::from_str(&data)?;
    let conv = &conversations[conv_idx];
    
    let sample_id = conv["sample_id"].as_str().unwrap();
    let workspace_name = format!("locomo-conv-{}", conv_idx);
    
    println!("\n{}", "=".repeat(60));
    println!("Evaluating {} (HYBRID SEARCH)", sample_id);
    println!("{}", "=".repeat(60));
    
    // Load workspace
    let workspace_mgr = WorkspaceManager::new(ModelType::BgeSmall)?;
    let memory_system = workspace_mgr.get_or_create_workspace(&workspace_name)?;
    memory_system.load_model()?;  // Load the model!
    
    // Get workspace ID
    let workspace_id: i64 = memory_system.database().execute(|conn| {
        Ok(conn.query_row(
            "SELECT id FROM workspaces WHERE name = ?1",
            [&workspace_name],
            |row| row.get(0),
        )?)
    })?;
    
    // Create hybrid retrieval engine
    let hybrid_engine = HybridRetrievalEngine::with_embedder(
        memory_system.database().clone(),
        memory_system.embedder()
    );
    
    let cat_names: HashMap<&str, &str> = [
        ("1", "Single-hop"),
        ("2", "Multi-hop"),
        ("3", "Temporal"),
        ("4", "Commonsense"),
        ("5", "Adversarial"),
    ].iter().cloned().collect();
    
    let mut results: HashMap<String, Vec<(f64, f64, f64, f64, f64)>> = HashMap::new();
    
    let qa_pairs = conv["qa"].as_array().unwrap();
    for (i, qa) in qa_pairs.iter().enumerate() {
        let question = qa["question"].as_str().unwrap();
        let category = qa["category"].to_string();
        let evidence: Vec<String> = qa.get("evidence")
            .and_then(|e| e.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        
        if evidence.is_empty() {
            continue;
        }
        
        // Use hybrid search
        let search_results = hybrid_engine.hybrid_search(question, workspace_id, 10)?;
        
        // Extract dialog IDs from results
        let retrieved_ids: Vec<String> = search_results.iter()
            .filter_map(|r| extract_dia_id(&r.content))
            .collect();
        
        let r1 = calculate_recall_at_k(&retrieved_ids, &evidence, 1);
        let r3 = calculate_recall_at_k(&retrieved_ids, &evidence, 3);
        let r5 = calculate_recall_at_k(&retrieved_ids, &evidence, 5);
        let r10 = calculate_recall_at_k(&retrieved_ids, &evidence, 10);
        let mrr = calculate_mrr(&retrieved_ids, &evidence);
        
        results.entry(category).or_insert_with(Vec::new).push((r1, r3, r5, r10, mrr));
        
        if (i + 1) % 20 == 0 {
            println!("  Processed {}/{} questions...", i + 1, qa_pairs.len());
        }
    }
    
    println!("\n{}", "=".repeat(60));
    println!("RESULTS");
    println!("{}", "=".repeat(60));
    
    let mut all_metrics = Vec::new();
    
    for cat in ["1", "2", "3", "4", "5"] {
        if let Some(metrics) = results.get(cat) {
            let count = metrics.len();
            let avg_r1 = metrics.iter().map(|m| m.0).sum::<f64>() / count as f64;
            let avg_r3 = metrics.iter().map(|m| m.1).sum::<f64>() / count as f64;
            let avg_r5 = metrics.iter().map(|m| m.2).sum::<f64>() / count as f64;
            let avg_r10 = metrics.iter().map(|m| m.3).sum::<f64>() / count as f64;
            let avg_mrr = metrics.iter().map(|m| m.4).sum::<f64>() / count as f64;
            
            println!("\n{}:", cat_names[cat]);
            println!("  Questions: {}", count);
            println!("  Recall@1:  {:.3}", avg_r1);
            println!("  Recall@3:  {:.3}", avg_r3);
            println!("  Recall@5:  {:.3}", avg_r5);
            println!("  Recall@10: {:.3}", avg_r10);
            println!("  MRR:       {:.3}", avg_mrr);
            
            all_metrics.push((avg_r1, avg_r3, avg_r5, avg_r10, avg_mrr));
        }
    }
    
    if !all_metrics.is_empty() {
        let total = all_metrics.len() as f64;
        let overall_r1 = all_metrics.iter().map(|m| m.0).sum::<f64>() / total;
        let overall_r3 = all_metrics.iter().map(|m| m.1).sum::<f64>() / total;
        let overall_r5 = all_metrics.iter().map(|m| m.2).sum::<f64>() / total;
        let overall_r10 = all_metrics.iter().map(|m| m.3).sum::<f64>() / total;
        let overall_mrr = all_metrics.iter().map(|m| m.4).sum::<f64>() / total;
        
        println!("\nOverall:");
        println!("  Questions: {}", results.values().map(|v| v.len()).sum::<usize>());
        println!("  Recall@1:  {:.3}", overall_r1);
        println!("  Recall@3:  {:.3}", overall_r3);
        println!("  Recall@5:  {:.3}", overall_r5);
        println!("  Recall@10: {:.3}", overall_r10);
        println!("  MRR:       {:.3}", overall_mrr);
    }
    
    Ok(())
}
