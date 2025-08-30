use crate::intelligence::{
    IntelligentFileAnalyzer, IntelligenceConfig,
    HealthStatus, CachePriority
};
use std::path::Path;
use colored::*;
use anyhow::Result;

pub fn analyze_repository(repo_path: Option<String>, detailed: bool) -> Result<()> {
    let path = repo_path.unwrap_or_else(|| ".".to_string());
    let repo_path = Path::new(&path);
    
    if !repo_path.exists() {
        anyhow::bail!("Repository path '{}' does not exist", path);
    }

    println!("{}", "🧠 Rune Intelligence Engine".cyan().bold());
    println!("{}", "Analyzing repository...".cyan());

    let mut analyzer = IntelligentFileAnalyzer::new();
    
    // Analyze repository
    let insights = analyzer.analyze_repository(repo_path)?;
    
    if detailed {
        insights.display_comprehensive_report();
    } else {
        // Display summary
        println!("\n{}", "📊 Repository Summary".yellow().bold());
        println!("  Project Type: {:?}", insights.project_type);
        println!("  Quality Score: {:.1}/100", insights.quality_score);
        println!("  Total Files: {}", insights.code_metrics.total_files);
        println!("  Lines of Code: {}", insights.code_metrics.lines_of_code);
        
        // Show top health indicators
        if !insights.health_indicators.is_empty() {
            println!("\n{}", "🏥 Key Health Indicators".yellow().bold());
            for indicator in insights.health_indicators.iter().take(3) {
                let status_color = match indicator.status {
                    HealthStatus::Excellent => "excellent".green(),
                    HealthStatus::Good => "good".blue(),
                    HealthStatus::Warning => "warning".yellow(),
                    HealthStatus::Critical => "critical".red(),
                };
                println!("  {} [{}]: {}", indicator.indicator, status_color, indicator.description);
            }
        }

        // Show top optimization suggestions
        if !insights.optimization_suggestions.is_empty() {
            println!("\n{}", "💡 Top Optimization Suggestions".yellow().bold());
            for suggestion in insights.optimization_suggestions.iter().take(3) {
                println!("  • [Impact: {:.1}] {}", suggestion.impact_score, suggestion.suggestion);
            }
        }
        
        println!("\n{}", "Use --detailed for comprehensive analysis".dimmed());
    }

    Ok(())
}

pub fn generate_predictions(repo_path: Option<String>) -> Result<()> {
    let path = repo_path.unwrap_or_else(|| ".".to_string());
    let repo_path = Path::new(&path);
    
    if !repo_path.exists() {
        anyhow::bail!("Repository path '{}' does not exist", path);
    }

    println!("{}", "🔮 Rune Predictive Insights".cyan().bold());
    println!("{}", "Generating predictions...".cyan());

    let mut analyzer = IntelligentFileAnalyzer::new();
    
    // First analyze the repository to build context
    let _insights = analyzer.analyze_repository(repo_path)?;
    
    // Generate predictive insights
    let predictions = analyzer.generate_predictive_insights(repo_path);
    
    if predictions.is_empty() {
        println!("{}", "No specific predictions available at this time.".yellow());
        println!("{}", "Analyzing more commits will improve prediction accuracy.".dimmed());
    } else {
        println!("\n{}", "🔮 Predictive Insights".yellow().bold());
        for prediction in &predictions {
            prediction.display();
        }
    }

    // Show caching suggestions
    let cache_suggestions = analyzer.get_smart_caching_suggestions();
    if !cache_suggestions.is_empty() {
        println!("\n{}", "⚡ Smart Caching Suggestions".yellow().bold());
        for suggestion in cache_suggestions.iter().take(5) {
            let priority_color = match suggestion.cache_priority {
                CachePriority::Critical => "critical".red(),
                CachePriority::High => "high".yellow(),
                CachePriority::Medium => "medium".blue(),
                CachePriority::Low => "low".dimmed(),
            };
            println!("  {} [{}] Access likelihood: {:.0}%", 
                suggestion.file_path, 
                priority_color, 
                suggestion.likelihood_access * 100.0
            );
        }
    }

    Ok(())
}

pub fn analyze_file(file_path: String) -> Result<()> {
    if !Path::new(&file_path).exists() {
        anyhow::bail!("File '{}' does not exist", file_path);
    }

    println!("{}", "🧠 Analyzing file...".cyan().bold());

    let mut analyzer = IntelligentFileAnalyzer::new();
    let analysis = analyzer.analyze_file(&file_path)?;
    
    analyzer.display_analysis(&analysis);
    
    // Update access pattern for predictive modeling
    analyzer.update_access_pattern(&file_path);
    
    Ok(())
}

pub fn configure_intelligence(
    enable: Option<bool>,
    lfs_threshold: Option<u64>,
    security_analysis: Option<bool>,
    performance_insights: Option<bool>,
    predictive_modeling: Option<bool>,
    repository_health: Option<bool>,
    code_quality: Option<bool>,
) -> Result<()> {
    println!("{}", "⚙️ Configuring Intelligence Engine".cyan().bold());

    let config_path = ".rune/intelligence.json";
    
    // Load existing config or create default
    let mut config = if Path::new(config_path).exists() {
        let content = std::fs::read_to_string(config_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        IntelligenceConfig::default()
    };

    // Apply updates
    if let Some(enabled) = enable {
        config.enabled = enabled;
        println!("  Intelligence Engine: {}", if enabled { "enabled".green() } else { "disabled".red() });
    }

    if let Some(threshold) = lfs_threshold {
        config.lfs_threshold_mb = threshold;
        println!("  LFS Threshold: {} MB", threshold);
    }

    if let Some(security) = security_analysis {
        config.features.security_analysis = security;
        println!("  Security Analysis: {}", if security { "enabled".green() } else { "disabled".red() });
    }

    if let Some(performance) = performance_insights {
        config.features.performance_insights = performance;
        println!("  Performance Insights: {}", if performance { "enabled".green() } else { "disabled".red() });
    }

    if let Some(predictive) = predictive_modeling {
        config.features.predictive_modeling = predictive;
        println!("  Predictive Modeling: {}", if predictive { "enabled".green() } else { "disabled".red() });
    }

    if let Some(health) = repository_health {
        config.features.repository_health = health;
        println!("  Repository Health: {}", if health { "enabled".green() } else { "disabled".red() });
    }

    if let Some(quality) = code_quality {
        config.features.code_quality_assessment = quality;
        println!("  Code Quality Assessment: {}", if quality { "enabled".green() } else { "disabled".red() });
    }

    // Ensure .rune directory exists
    std::fs::create_dir_all(".rune")?;
    
    // Save config
    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path, content)?;
    
    println!("\n{}", "✅ Intelligence configuration saved".green());
    
    Ok(())
}

pub fn show_intelligence_status() -> Result<()> {
    let config_path = ".rune/intelligence.json";
    
    let config = if Path::new(config_path).exists() {
        let content = std::fs::read_to_string(config_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        IntelligenceConfig::default()
    };

    println!("{}", "🧠 Intelligence Engine Status".cyan().bold());
    println!("{}", "═".repeat(40).cyan());
    
    println!("Engine Status: {}", if config.enabled { "enabled".green() } else { "disabled".red() });
    println!("LFS Threshold: {} MB", config.lfs_threshold_mb);
    println!("Analysis Depth: {:?}", config.analysis_depth);
    println!("Notification Level: {:?}", config.notification_level);
    
    println!("\n{}", "Features".yellow().bold());
    println!("  Security Analysis: {}", if config.features.security_analysis { "✓".green() } else { "✗".red() });
    println!("  Performance Insights: {}", if config.features.performance_insights { "✓".green() } else { "✗".red() });
    println!("  Predictive Modeling: {}", if config.features.predictive_modeling { "✓".green() } else { "✗".red() });
    println!("  Repository Health: {}", if config.features.repository_health { "✓".green() } else { "✗".red() });
    println!("  Code Quality Assessment: {}", if config.features.code_quality_assessment { "✓".green() } else { "✗".red() });
    
    Ok(())
}
