use anyhow::Result;
use colored::*;
use std::collections::HashMap;

pub mod content;
pub mod server;
pub mod templates;
pub mod examples;
pub mod tutorial;

/// Main documentation engine that handles all documentation-related functionality
pub struct DocsEngine {
    content: HashMap<String, String>,
    examples: HashMap<String, Vec<Example>>,
    tutorials: Vec<Tutorial>,
}

/// Represents a code example with explanation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Example {
    pub title: String,
    pub description: String,
    pub commands: Vec<String>,
    pub expected_output: Option<String>,
    pub category: String,
}

/// Represents an interactive tutorial
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tutorial {
    pub id: String,
    pub title: String,
    pub description: String,
    pub steps: Vec<TutorialStep>,
    pub difficulty: TutorialDifficulty,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TutorialStep {
    pub title: String,
    pub instruction: String,
    pub command: Option<String>,
    pub expected_files: Option<Vec<String>>,
    pub verification: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TutorialDifficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl DocsEngine {
    /// Create a new documentation engine
    pub fn new() -> Result<Self> {
        let mut engine = DocsEngine {
            content: HashMap::new(),
            examples: HashMap::new(),
            tutorials: Vec::new(),
        };
        
        engine.load_content()?;
        engine.load_examples()?;
        engine.load_tutorials()?;
        
        Ok(engine)
    }
    
    /// Load all embedded documentation content
    fn load_content(&mut self) -> Result<()> {
        // Load documentation content (simplified for now)
        self.content.insert("getting-started".to_string(), include_str!("../content/getting-started.md").to_string());
        
        // For now, create the other content as placeholders
        self.content.insert("commands".to_string(), "# Commands\n\nCommand documentation coming soon...".to_string());
        self.content.insert("migration".to_string(), "# Migration\n\nMigration guide coming soon...".to_string());
        self.content.insert("best-practices".to_string(), "# Best Practices\n\nBest practices guide coming soon...".to_string());
        self.content.insert("troubleshooting".to_string(), "# Troubleshooting\n\nTroubleshooting guide coming soon...".to_string());
        
        Ok(())
    }
    
    /// Load all code examples
    fn load_examples(&mut self) -> Result<()> {
        self.examples = examples::load_all_examples();
        Ok(())
    }
    
    /// Load all tutorials
    fn load_tutorials(&mut self) -> Result<()> {
        self.tutorials = tutorial::load_all_tutorials();
        Ok(())
    }
    
    /// Get content by key
    pub fn get_content(&self, key: &str) -> Option<&String> {
        self.content.get(key)
    }
    
    /// Get topic content (alias for get_content)
    pub fn get_topic_content(&self, topic: &str) -> Result<String> {
        self.content.get(topic)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Topic '{}' not found", topic))
    }
    
    /// Get examples by category name
    pub fn get_examples_by_category(&self, category: &str) -> Vec<&Example> {
        self.examples
            .get(category)
            .map(|examples| examples.iter().collect())
            .unwrap_or_default()
    }
    
    /// Search examples
    pub fn search_examples(&self, query: &str) -> Vec<&Example> {
        let query_lower = query.to_lowercase();
        self.examples
            .values()
            .flatten()
            .filter(|ex| 
                ex.title.to_lowercase().contains(&query_lower) ||
                ex.description.to_lowercase().contains(&query_lower) ||
                ex.category.to_lowercase().contains(&query_lower)
            )
            .collect()
    }
    
    /// Get example by name
    pub fn get_example_by_name(&self, name: &str) -> Option<&Example> {
        self.examples
            .values()
            .flatten()
            .find(|ex| ex.title.to_lowercase() == name.to_lowercase())
    }
    
    /// Get all examples
    pub fn get_all_examples(&self) -> Vec<&Example> {
        self.examples.values().flatten().collect()
    }
    
    /// Start documentation server
    pub async fn start_server(&self, addr: &str) -> Result<()> {
        // Parse the address to extract port
        let port = addr.split(':').last()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);
            
        // For now, create a basic implementation
        println!("🌐 Documentation server would start on port {} (coming soon!)", port);
        println!("📚 Visit http://{} to view documentation", addr);
        Ok(())
    }
    
    /// Run interactive tutorial
    pub async fn run_interactive_tutorial(&self, _tutorial: &Tutorial) -> Result<()> {
        println!("🎓 Interactive tutorial support coming soon!");
        println!("For now, please refer to the documentation examples.");
        Ok(())
    }
    
    /// Resume tutorial
    pub async fn resume_tutorial(&self, _tutorial: &Tutorial) -> Result<()> {
        println!("🔄 Tutorial resume support coming soon!");
        println!("For now, please restart the tutorial.");
        Ok(())
    }
    
    /// Get examples by category
    pub fn get_examples(&self, category: Option<&str>) -> Vec<&Example> {
        match category {
            Some(cat) => self.examples
                .values()
                .flatten()
                .filter(|ex| ex.category == cat)
                .collect(),
            None => self.examples.values().flatten().collect(),
        }
    }
    
    /// Get all tutorials
    pub fn get_tutorials(&self) -> &[Tutorial] {
        &self.tutorials
    }
    
    /// Get tutorial by ID
    pub fn get_tutorial(&self, id: &str) -> Option<&Tutorial> {
        self.tutorials.iter().find(|t| t.id == id)
    }
    
    /// Search content and examples
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        
        // Search content
        for (key, content) in &self.content {
            if content.to_lowercase().contains(&query_lower) {
                results.push(SearchResult {
                    title: key.clone(),
                    snippet: extract_snippet(content, &query_lower),
                    result_type: SearchResultType::Documentation,
                    url: format!("/{}", key),
                });
            }
        }
        
        // Search examples
        for examples in self.examples.values() {
            for example in examples {
                if example.title.to_lowercase().contains(&query_lower) ||
                   example.description.to_lowercase().contains(&query_lower) {
                    results.push(SearchResult {
                        title: example.title.clone(),
                        snippet: example.description.clone(),
                        result_type: SearchResultType::Example,
                        url: format!("/examples/{}", example.category),
                    });
                }
            }
        }
        
        results
    }
    
    /// Show enhanced help for a specific command
    pub fn show_command_help(&self, command: &str) -> Result<()> {
        println!("{}", format!("📚 Enhanced Help: rune {}", command).bold().blue());
        println!();
        
        // Show basic help first
        if let Some(help_content) = self.get_content(&format!("commands/{}", command)) {
            println!("{}", help_content);
        }
        
        // Show examples
        let examples = self.get_examples(Some(command));
        if !examples.is_empty() {
            println!("{}", "💡 Examples:".bold().yellow());
            for example in examples.iter().take(3) {
                println!("  {} {}", "▶".green(), example.title.bold());
                println!("    {}", example.description.dimmed());
                for cmd in &example.commands {
                    println!("    {} {}", "$".blue(), cmd.cyan());
                }
                println!();
            }
            
            if examples.len() > 3 {
                println!("  {} {}", "💡".yellow(), 
                    format!("See more examples with: rune examples {}", command).dimmed());
            }
        }
        
        Ok(())
    }
    
    /// Show examples for a category
    pub fn show_examples(&self, category: Option<&str>) -> Result<()> {
        match category {
            Some(cat) => {
                println!("{}", format!("💡 Examples: {}", cat).bold().green());
                let examples = self.get_examples(Some(cat));
                if examples.is_empty() {
                    println!("No examples found for category: {}", cat);
                    return Ok(());
                }
                
                for example in examples {
                    self.display_example(example);
                }
            }
            None => {
                println!("{}", "💡 All Examples".bold().green());
                println!("Available categories:");
                
                let mut categories: Vec<_> = self.examples.keys().collect();
                categories.sort();
                
                for cat in categories {
                    let count = self.examples[cat].len();
                    println!("  {} {} ({} examples)", "📁".blue(), cat.bold(), count.to_string().dimmed());
                }
                
                println!();
                println!("{}", "Use 'rune examples <category>' to see specific examples".dimmed());
            }
        }
        
        Ok(())
    }
    
    /// Display a single example
    fn display_example(&self, example: &Example) {
        println!();
        println!("{} {}", "▶".green().bold(), example.title.bold());
        println!("  {}", example.description);
        println!();
        
        for (i, cmd) in example.commands.iter().enumerate() {
            println!("  {}. {} {}", i + 1, "$".blue(), cmd.cyan());
        }
        
        if let Some(output) = &example.expected_output {
            println!();
            println!("  {} Expected output:", "📋".yellow());
            for line in output.lines() {
                println!("  {}", line.dimmed());
            }
        }
        
        println!("{}", "─".repeat(60).dimmed());
    }
}

/// Search result structure
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub result_type: SearchResultType,
    pub url: String,
}

#[derive(Debug, Clone)]
pub enum SearchResultType {
    Documentation,
    Example,
    Tutorial,
}

/// Extract a snippet around the search query
fn extract_snippet(content: &str, query: &str) -> String {
    let query_pos = content.to_lowercase().find(query);
    if let Some(pos) = query_pos {
        let start = pos.saturating_sub(50);
        let end = std::cmp::min(pos + query.len() + 50, content.len());
        format!("...{}", &content[start..end])
    } else {
        content.chars().take(100).collect::<String>() + "..."
    }
}

impl Default for DocsEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| DocsEngine {
            content: HashMap::new(),
            examples: HashMap::new(),
            tutorials: Vec::new(),
        })
    }
}
