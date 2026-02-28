use agent_memory_rs::{MemorySystem, storage::memory_store::Memory, ModelType, WorkspaceManager};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

// Parse LoCoMo timestamp format: "1:56 pm on 8 May, 2023"
fn parse_locomo_timestamp(datetime_str: &str) -> Option<String> {
    use chrono::{NaiveDateTime, TimeZone, Utc};
    
    // Parse format: "1:56 pm on 8 May, 2023"
    let parts: Vec<&str> = datetime_str.split(" on ").collect();
    if parts.len() != 2 {
        return None;
    }
    
    let time_str = parts[0];
    let date_str = parts[1];
    
    // Parse date: "8 May, 2023"
    let date_parts: Vec<&str> = date_str.split(", ").collect();
    if date_parts.len() != 2 {
        return None;
    }
    
    let day_month: Vec<&str> = date_parts[0].split(' ').collect();
    if day_month.len() != 2 {
        return None;
    }
    
    let day = day_month[0].parse::<u32>().ok()?;
    let month_str = day_month[1];
    let year = date_parts[1].parse::<i32>().ok()?;
    
    let month = match month_str {
        "January" | "Jan" => 1, "February" | "Feb" => 2, "March" | "Mar" => 3,
        "April" | "Apr" => 4, "May" => 5, "June" | "Jun" => 6,
        "July" | "Jul" => 7, "August" | "Aug" => 8, "September" | "Sep" => 9,
        "October" | "Oct" => 10, "November" | "Nov" => 11, "December" | "Dec" => 12,
        _ => return None,
    };
    
    // Parse time: "1:56 pm"
    let time_parts: Vec<&str> = time_str.split(':').collect();
    if time_parts.len() != 2 {
        return None;
    }
    
    let mut hour = time_parts[0].trim().parse::<u32>().ok()?;
    let min_ampm: Vec<&str> = time_parts[1].trim().split(' ').collect();
    if min_ampm.len() != 2 {
        return None;
    }
    
    let minute = min_ampm[0].parse::<u32>().ok()?;
    let ampm = min_ampm[1].to_lowercase();
    
    if ampm == "pm" && hour != 12 {
        hour += 12;
    } else if ampm == "am" && hour == 12 {
        hour = 0;
    }
    
    let naive_dt = NaiveDateTime::parse_from_str(
        &format!("{}-{:02}-{:02} {:02}:{:02}:00", year, month, day, hour, minute),
        "%Y-%m-%d %H:%M:%S"
    ).ok()?;
    
    Some(Utc.from_utc_datetime(&naive_dt).to_rfc3339())
}

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
        
        // Get session timestamp
        let datetime_key = format!("session_{}_date_time", session_num);
        let session_datetime = conversation.get(&datetime_key)
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let session_timestamp = parse_locomo_timestamp(session_datetime);
        
        let turns = conversation[&session_key].as_array().unwrap();
        
        for turn in turns {
            let dia_id = turn["dia_id"].as_str().unwrap();
            let speaker = turn["speaker"].as_str().unwrap();
            let text = turn["text"].as_str().unwrap();
            
            // Add minimal timestamp at the END (just month/year)
            let timestamp_suffix = if let Some(ref ts) = session_timestamp {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                    format!(" ({})", dt.format("%b'%y"))  // May'23
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            
            let memory_text = format!("[{}] {}: {}{}", dia_id, speaker, text, timestamp_suffix);
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
                created_at: session_timestamp.clone(),
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
