use agent_memory_rs::cli::MemoryCLI;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = MemoryCLI::new("memory.db")?;

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: agent-memory-cli <command> [options]");
        println!("\nCommands:");
        println!("  workspace create --name NAME --path PATH");
        println!("  workspace list");
        println!("  workspace delete --id ID");
        println!(
            "  store --workspace ID --type TYPE --context TEXT [--outcome TEXT] [--valence FLOAT]"
        );
        println!("  stats --workspace ID");
        println!("  query --workspace ID <text> [--limit N]");
        return Ok(());
    }

    match args[1].as_str() {
        "workspace" => {
            let subcommand = args.get(2).expect("workspace subcommand required");
            match subcommand.as_str() {
                "create" => {
                    let name = args
                        .iter()
                        .position(|a| a == "--name")
                        .and_then(|i| args.get(i + 1))
                        .expect("--name required");
                    let path = args
                        .iter()
                        .position(|a| a == "--path")
                        .and_then(|i| args.get(i + 1))
                        .expect("--path required");
                    cli.create_workspace(name, path)?;
                }
                "list" => {
                    cli.list_workspaces()?;
                }
                "delete" => {
                    let id = args
                        .iter()
                        .position(|a| a == "--id")
                        .and_then(|i| args.get(i + 1))
                        .and_then(|s| s.parse().ok())
                        .expect("--id required");
                    cli.delete_workspace(id)?;
                }
                _ => {
                    println!("Unknown workspace subcommand: {}", subcommand);
                }
            }
        }
        "store" => {
            let workspace_id = args
                .iter()
                .position(|a| a == "--workspace")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .expect("--workspace required");
            let event_type = args
                .iter()
                .position(|a| a == "--type")
                .and_then(|i| args.get(i + 1))
                .expect("--type required");
            let context = args
                .iter()
                .position(|a| a == "--context")
                .and_then(|i| args.get(i + 1))
                .expect("--context required");
            let outcome = args
                .iter()
                .position(|a| a == "--outcome")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());
            let valence = args
                .iter()
                .position(|a| a == "--valence")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok());
            cli.store_episode(workspace_id, event_type, context, outcome, valence)
                .await?;
        }
        "stats" => {
            let workspace_id = args
                .iter()
                .position(|a| a == "--workspace")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .expect("--workspace required");
            cli.stats(workspace_id)?;
        }
        "query" => {
            let workspace_id = args
                .iter()
                .position(|a| a == "--workspace")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .expect("--workspace required");

            // Find query text - skip command name, flags, and flag values
            let mut query_text = String::new();
            let mut skip_next = false;
            for (_i, arg) in args.iter().enumerate().skip(2) {
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

            let limit = args
                .iter()
                .position(|a| a == "--limit")
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
