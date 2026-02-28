#!/usr/bin/env python3
"""Quick LoCoMo benchmark using direct Rust API (no MCP overhead)."""

import json
import subprocess
import sys
from pathlib import Path

DATA = Path.home() / "newProject/LoCoMo/data/locomo10.json"

# Create a simple Rust benchmark program
BENCHMARK_CODE = '''
use agent_memory_rs::services::memory_manager::MemoryManager;
use agent_memory_rs::storage::database::Database;
use agent_memory_rs::models::dtos::Episode;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let data = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&data).unwrap();
    let conv = &json[0]; // First conversation
    
    // Setup
    let db = Database::new(":memory:").unwrap();
    db.execute(|conn| {
        conn.execute("INSERT INTO workspaces (id, name, path, created_at) VALUES (1, 'locomo', '/tmp', datetime('now'))", []).unwrap();
        Ok(())
    }).unwrap();
    let manager = MemoryManager::new(db);
    
    // Load conversation
    let conversation = &conv["conversation"];
    let mut loaded = 0;
    
    for i in 1..=20 {
        let key = format!("session_{}", i);
        if let Some(turns) = conversation.get(&key).and_then(|v| v.as_array()) {
            for turn in turns {
                let dia_id = turn["dia_id"].as_str().unwrap();
                let speaker = turn["speaker"].as_str().unwrap();
                let text = turn["text"].as_str().unwrap();
                
                let episode = Episode {
                    id: None,
                    workspace_id: 1,
                    agent_id: None,
                    timestamp: chrono::Local::now().to_rfc3339(),
                    conversation_id: None,
                    event_type: "dialog".to_string(),
                    context: serde_json::json!({"text": format!("[{}] {}: {}", dia_id, speaker, text)}),
                    outcome: None,
                    valence: None,
                    archived: false,
                    created_at: None,
                };
                
                manager.store_episode(episode).await.unwrap();
                loaded += 1;
            }
        }
    }
    
    println!("Loaded {} turns", loaded);
    
    // Evaluate
    let qa_pairs = conv["qa"].as_array().unwrap();
    let mut results: HashMap<String, Vec<(bool, bool, bool, bool, f64)>> = HashMap::new();
    
    for qa in qa_pairs {
        let question = qa["question"].as_str().unwrap();
        let category = qa["category"].as_str().unwrap().to_string();
        let evidence: Vec<String> = qa.get("evidence")
            .and_then(|e| e.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        
        if evidence.is_empty() {
            continue;
        }
        
        // Search using BM25 only (fast)
        let search_results = manager.retrieve(question, 1, 10).unwrap();
        
        let mut retrieved_ids = Vec::new();
        for r in &search_results {
            if let Some(start) = r.content.find("[D") {
                if let Some(end) = r.content[start..].find(']') {
                    retrieved_ids.push(r.content[start+1..start+end].to_string());
                }
            }
        }
        
        let r1 = evidence.iter().any(|e| retrieved_ids.get(0).map_or(false, |r| r == e));
        let r3 = evidence.iter().any(|e| retrieved_ids.iter().take(3).any(|r| r == e));
        let r5 = evidence.iter().any(|e| retrieved_ids.iter().take(5).any(|r| r == e));
        let r10 = evidence.iter().any(|e| retrieved_ids.iter().take(10).any(|r| r == e));
        
        let mrr = retrieved_ids.iter().enumerate()
            .find(|(_, r)| evidence.contains(r))
            .map(|(i, _)| 1.0 / (i + 1) as f64)
            .unwrap_or(0.0);
        
        results.entry(category).or_default().push((r1, r3, r5, r10, mrr));
    }
    
    // Print results
    println!("\\n{}", "=".repeat(70));
    println!("{:<15} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}", "Category", "Count", "R@1", "R@3", "R@5", "R@10", "MRR");
    println!("{}", "=".repeat(70));
    
    for cat in ["Factual", "Temporal", "Causal", "Hypothetical"] {
        if let Some(metrics) = results.get(cat) {
            let count = metrics.len();
            let r1 = metrics.iter().filter(|m| m.0).count() as f64 / count as f64 * 100.0;
            let r3 = metrics.iter().filter(|m| m.1).count() as f64 / count as f64 * 100.0;
            let r5 = metrics.iter().filter(|m| m.2).count() as f64 / count as f64 * 100.0;
            let r10 = metrics.iter().filter(|m| m.3).count() as f64 / count as f64 * 100.0;
            let mrr = metrics.iter().map(|m| m.4).sum::<f64>() / count as f64;
            
            println!("{:<15} {:>8} {:>7.1}% {:>7.1}% {:>7.1}% {:>7.1}% {:>8.3}", cat, count, r1, r3, r5, r10, mrr);
        }
    }
    
    println!("{}", "=".repeat(70));
}
'''

def main():
    if not DATA.exists():
        print(f"❌ Data not found: {DATA}")
        sys.exit(1)
    
    print("🚀 LoCoMo Benchmark - Direct Rust API")
    print("=" * 70)
    
    # Write benchmark program
    bench_file = Path("/tmp/locomo_bench.rs")
    bench_file.write_text(BENCHMARK_CODE)
    
    # Compile and run
    print("\n📦 Compiling benchmark...")
    result = subprocess.run([
        "rustc",
        "--edition", "2021",
        "-L", "target/release/deps",
        "--extern", "agent_memory_rs=target/release/libagent_memory_rs.rlib",
        "--extern", "tokio=target/release/deps/libtokio.rlib",
        "--extern", "serde_json=target/release/deps/libserde_json.rlib",
        "--extern", "chrono=target/release/deps/libchrono.rlib",
        str(bench_file),
        "-o", "/tmp/locomo_bench"
    ], capture_output=True, text=True, cwd="<project-root>")
    
    if result.returncode != 0:
        print(f"❌ Compilation failed:\n{result.stderr}")
        sys.exit(1)
    
    print("✅ Compiled\n")
    print("🔍 Running benchmark...\n")
    
    # Run
    result = subprocess.run(["/tmp/locomo_bench", str(DATA)], capture_output=True, text=True)
    print(result.stdout)
    if result.stderr:
        print(result.stderr)

if __name__ == '__main__':
    main()
