use crate::services::memory_manager::MemoryManager;
use anyhow::Result;

pub struct ContextInjectionService {
    manager: MemoryManager,
}

impl ContextInjectionService {
    pub fn new(manager: MemoryManager) -> Self {
        Self { manager }
    }

    pub fn prepare_context(&self, query: &str, workspace_id: i64, budget: usize) -> Result<String> {
        let mut context = String::new();
        let mut tokens_used = 0;

        // Budget allocation: synopsis 25%, semantic 40%, episodic 25%, procedural 10%
        let synopsis_budget = budget / 4;
        let semantic_budget = (budget * 2) / 5;
        let episodic_budget = budget / 4;
        let procedural_budget = budget / 10;

        // Load synopsis if available
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if let Ok(Some(synopsis)) = self.manager.get_synopsis(workspace_id, &today) {
            let synopsis_text = self.format_synopsis(&synopsis);
            let synopsis_tokens = self.estimate_tokens(&synopsis_text);
            
            if tokens_used + synopsis_tokens <= synopsis_budget {
                context.push_str(&synopsis_text);
                tokens_used += synopsis_tokens;
            }
        }

        // Load relevant memories
        if let Ok(results) = self.manager.retrieve_hierarchical(query, workspace_id, 20) {
            for result in results {
                let formatted = self.format_memory(&result.memory_type, &result.content, result.score);
                let tokens = self.estimate_tokens(&formatted);
                
                let type_budget = match result.memory_type.as_str() {
                    "semantic" => semantic_budget,
                    "episodic" => episodic_budget,
                    "procedural" => procedural_budget,
                    _ => budget / 10,
                };

                if tokens_used + tokens <= budget && tokens <= type_budget {
                    context.push_str(&formatted);
                    tokens_used += tokens;
                }
            }
        }

        Ok(context)
    }

    fn format_synopsis(&self, synopsis: &crate::models::dtos::Synopsis) -> String {
        format!(
            "\n## Daily Synopsis ({})\n{}\n\nKey Insights:\n{}\n\n",
            synopsis.date,
            synopsis.summary,
            synopsis.key_insights.iter()
                .enumerate()
                .map(|(i, insight)| format!("{}. {}", i + 1, insight))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn format_memory(&self, memory_type: &str, content: &str, score: f64) -> String {
        format!(
            "\n[{}] (relevance: {:.2})\n{}\n",
            memory_type.to_uppercase(),
            score,
            content
        )
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        // Simple estimation: ~4 chars per token
        (text.len() + 3) / 4
    }
}
