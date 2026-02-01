use agent_memory_rs::cli::MemoryCLI;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = MemoryCLI::new("memory.db")?;
    
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: agent-memory-cli <command> [options]");
        println!("\nCommands:");
        println!("  consolidate --date YYYY-MM-DD");
        println!("  synopsis --workspace ID --date YYYY-MM-DD");
        println!("  stats --workspace ID");
        println!("  prune --workspace ID --threshold FLOAT [--dry-run]");
        println!("  query --workspace ID <text> [--limit N]");
        return Ok(());
    }
    
    match args[1].as_str() {
        "consolidate" => {
            let date = args.iter().position(|a| a == "--date")
                .and_then(|i| args.get(i + 1))
                .expect("--date required").to_string();
            cli.consolidate(date).await?;
        }
        "synopsis" => {
            let workspace_id = args.iter().position(|a| a == "--workspace")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .expect("--workspace required");
            let date = args.iter().position(|a| a == "--date")
                .and_then(|i| args.get(i + 1))
                .expect("--date required");
            cli.synopsis(workspace_id, date)?;
        }
        "stats" => {
            let workspace_id = args.iter().position(|a| a == "--workspace")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .expect("--workspace required");
            cli.stats(workspace_id)?;
        }
        "prune" => {
            let workspace_id = args.iter().position(|a| a == "--workspace")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .expect("--workspace required");
            let threshold = args.iter().position(|a| a == "--threshold")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.3);
            let dry_run = args.contains(&"--dry-run".to_string());
            cli.prune(workspace_id, dry_run).await?;
        }
        "query" => {
            let workspace_id = args.iter().position(|a| a == "--workspace")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .expect("--workspace required");
            
            // Find query text - skip command name, flags, and flag values
            let mut query_text = String::new();
            let mut skip_next = false;
            for (i, arg) in args.iter().enumerate().skip(2) {
                if skip_next {
                    skip_next = false;
                    continue;
                }
                if arg.starts_with("--") {
                    skip_next = true; // Skip the flag's value
                    continue;
                }
                if !query_text.is_empty() {
                    query_text.push(' ');
                }
                query_text.push_str(arg);
            }
            
            if query_text.is_empty() {
                eprintln!("Error: query text required");
                return Ok(());
            }
            
            let limit = args.iter().position(|a| a == "--limit")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            cli.query(workspace_id, &query_text, limit)?;
        }
        _ => {
            println!("Unknown command: {}", args[1]);
        }
    }
    
    Ok(())
}
