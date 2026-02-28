use agent_memory_rs::{MemorySystem, storage::memory_store::Memory, ModelType, WorkspaceManager};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <conversation_index>", args[0]);
        std::process::exit(1);
    }
    
    let conv_idx: usize = args[1].parse()?;
    
    // Get LoCoMo data path from env or use default
    let data_path = std::env::var("LOCOMO_DATA_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap())
                .join("LoCoMo/data/locomo10.json")
        });
    
    let data: Value = serde_json::from_str(&fs::read_to_string(data_path)?)?;
    let conversations = data.as_array().unwrap();
    
    if conv_idx >= conversations.len() {
        eprintln!("Error: Index must be 0-{}", conversations.len() - 1);
        std::process::exit(1);
    }
    
    let conv = &conversations[conv_idx];
    let sample_id = conv["sample_id"].as_str().unwrap();
    let workspace_name = format!("locomo-conv-{}", conv_idx);
    
    println!("\nLoading {} into workspace '{}'", sample_id, workspace_name);
    
    // Create workspace and memory system with BgeSmall model
    let workspace_mgr = WorkspaceManager::new(ModelType::BgeSmall)?;
    let mut memory_system = workspace_mgr.get_or_create_workspace(&workspace_name)?;
    
    // Load model
    memory_system.load_model()?;
    println!("Model loaded");
    
    // Get workspace ID
    let workspace_id: i64 = memory_system.database().execute(|conn| {
        Ok(conn.query_row(
            "SELECT id FROM workspaces WHERE name = ?1",
            [&workspace_name],
            |row| row.get(0),
        )?)
    })?;
    
    println!("Workspace ID: {}", workspace_id);
    
    // Get all sessions
    let conversation = conv["conversation"].as_object().unwrap();
    let mut sessions: Vec<String> = conversation.keys()
        .filter(|k| k.starts_with("session_") && !k.ends_with("_date_time"))
        .cloned()
        .collect();
    sessions.sort();
    
    println!("Found {} sessions", sessions.len());
    
    let mut memories_loaded = 0;
    for session_key in sessions {
        let session_num = session_key.replace("session_", "");
        let turns = conversation[&session_key].as_array().unwrap();
        
        for turn in turns {
            let dia_id = turn["dia_id"].as_str().unwrap();
            let speaker = turn["speaker"].as_str().unwrap();
            let text = turn["text"].as_str().unwrap();
            
            let memory_text = format!("[{}] {}: {}", dia_id, speaker, text);
            let tags = format!("locomo,{},session_{},{}", sample_id, session_num, dia_id);
            
            let memory = Memory {
                id: None,
                workspace_id,
                agent_id: None,
                text: memory_text,
                tags: Some(tags),
                importance_score: 0.5,
                access_count: 0,
                last_accessed: None,
                conversation_id: Some(sample_id.to_string()),
                parent_memory_id: None,
                user_feedback: None,
                source_episodes: vec![],
                confidence: 0.5,
                last_validated: None,
                created_at: None,
                updated_at: None,
            };
            
            memory_system.learn(&memory)?;
            memories_loaded += 1;
            
            if memories_loaded % 50 == 0 {
                println!("  Loaded {} turns...", memories_loaded);
            }
        }
    }
    
    println!("✅ Loaded {} turns from {}", memories_loaded, sample_id);
    Ok(())
}
