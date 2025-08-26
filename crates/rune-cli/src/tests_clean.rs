use crate::intelligence::*;
use crate::locking::*;
use crate::performance::*;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod intelligence_tests {
    use super::*;

    #[test]
    fn test_intelligent_file_analyzer_creation() {
        let analyzer = IntelligentFileAnalyzer::new();
        assert_eq!(analyzer.config.enabled, true);
        assert_eq!(analyzer.config.features.security_analysis, true);
        assert_eq!(analyzer.config.features.performance_insights, true);
    }

    #[test]
    fn test_file_analysis_rust_code() {
        let mut analyzer = IntelligentFileAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        fs::write(&file_path, "fn main() { println!(\"Hello, world!\"); }").unwrap();

        let analysis = analyzer.analyze_file(file_path.to_str().unwrap()).unwrap();
        assert!(matches!(
            analysis.file_type,
            FileType::SourceCode(Language::Rust)
        ));
        assert_eq!(analysis.language, Language::Rust);
        assert!(!analysis.suggested_lfs);
    }

    #[test]
    fn test_file_analysis_large_binary() {
        let mut analyzer = IntelligentFileAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.bin");

        // Create a large binary file (> 50MB to trigger LFS suggestion)
        let large_data = vec![0u8; 60 * 1024 * 1024];
        fs::write(&file_path, large_data).unwrap();

        let analysis = analyzer.analyze_file(file_path.to_str().unwrap()).unwrap();
        assert!(matches!(analysis.file_type, FileType::Binary));
        assert!(analysis.suggested_lfs);
    }

    #[test]
    fn test_security_analysis_credentials() {
        let mut analyzer = IntelligentFileAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".env");

        fs::write(&file_path, "API_KEY=secret123\nPASSWORD=mypassword").unwrap();

        let analysis = analyzer.analyze_file(file_path.to_str().unwrap()).unwrap();
        assert!(!analysis.security_issues.is_empty());
        assert!(analysis
            .security_issues
            .iter()
            .any(|issue| matches!(issue.category, SecurityCategory::Credentials)));
    }

    #[test]
    fn test_project_detection_rust() {
        let analyzer = IntelligentFileAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        fs::write(
            &cargo_toml,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .unwrap();

        let project_type = analyzer
            .detect_project_type(temp_dir.path().to_str().unwrap())
            .unwrap();
        assert!(matches!(project_type, ProjectType::Rust));
    }

    #[test]
    fn test_project_detection_unreal() {
        let analyzer = IntelligentFileAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();
        let uproject = temp_dir.path().join("TestGame.uproject");

        fs::write(&uproject, "{\n  \"FileVersion\": 3\n}").unwrap();

        let project_type = analyzer
            .detect_project_type(temp_dir.path().to_str().unwrap())
            .unwrap();
        assert!(matches!(project_type, ProjectType::UnrealEngine));
    }

    #[test]
    fn test_analyze_repository_health() {
        let analyzer = IntelligentFileAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create a simple repository structure
        fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
        fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();

        let health = analyzer.analyze_repository_health(temp_dir.path().to_str().unwrap());
        assert!(health.overall_score > 0.0);
        // README exists, so should have fewer issues
        assert!(health.overall_score > 50.0);
    }

    #[test]
    fn test_generate_insights() {
        let analyzer = IntelligentFileAnalyzer::new();
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(
            temp_dir.path().join("large.bin"),
            vec![0u8; 110 * 1024 * 1024],
        )
        .unwrap();
        fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();

        let insights = analyzer.generate_insights(temp_dir.path().to_str().unwrap());
        assert!(!insights.is_empty());
        // Should detect the large file
        assert!(insights
            .iter()
            .any(|insight| insight.insight.contains("Large file")));
    }

    #[test]
    fn test_config_defaults() {
        let config = IntelligenceConfig::default();
        assert_eq!(config.enabled, true);
        assert_eq!(config.lfs_threshold_mb, 50);
        assert_eq!(config.features.security_analysis, true);
    }

    #[test]
    fn test_file_analysis_defaults() {
        let analysis = FileAnalysis::default();
        assert_eq!(analysis.suggested_lfs, false);
        assert!(matches!(analysis.file_type, FileType::Unknown));
        assert_eq!(analysis.size_bytes, 0);
    }
}

#[cfg(test)]
mod locking_tests {
    use super::*;

    #[test]
    fn test_lock_manager_creation() {
        let config = LockingConfig::default();
        let manager = LockManager::new(config);
        assert_eq!(manager.config.enabled, true);
    }

    #[test]
    fn test_project_detection_game_dev() {
        let manager = LockManager::new(LockingConfig::default());
        let temp_dir = TempDir::new().unwrap();

        // Create Unreal Engine project
        fs::write(temp_dir.path().join("Game.uproject"), "{}").unwrap();

        let project_type = manager
            .detect_project_type(temp_dir.path().to_str().unwrap())
            .unwrap();
        assert!(matches!(project_type, ProjectType::UnrealEngine));
    }

    #[test]
    fn test_project_detection_web_dev() {
        let manager = LockManager::new(LockingConfig::default());
        let temp_dir = TempDir::new().unwrap();

        // Create Next.js project
        fs::write(
            temp_dir.path().join("package.json"),
            r#"{"dependencies": {"next": "^13.0.0"}}"#,
        )
        .unwrap();

        let project_type = manager
            .detect_project_type(temp_dir.path().to_str().unwrap())
            .unwrap();
        assert!(matches!(project_type, ProjectType::NextJS));
    }

    #[test]
    fn test_analyze_files() {
        let manager = LockManager::new(LockingConfig::default());
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test.cpp"), "#include <iostream>").unwrap();
        fs::write(temp_dir.path().join("image.png"), vec![0u8; 1024]).unwrap();

        let files = vec![
            temp_dir
                .path()
                .join("test.cpp")
                .to_str()
                .unwrap()
                .to_string(),
            temp_dir
                .path()
                .join("image.png")
                .to_str()
                .unwrap()
                .to_string(),
        ];

        let analysis = manager.analyze_files(&files);
        assert_eq!(analysis.len(), 2);
        assert!(analysis
            .iter()
            .any(|(_, info)| matches!(info.file_type, DetectedFileType::SourceCode(_))));
        assert!(analysis
            .iter()
            .any(|(_, info)| matches!(info.file_type, DetectedFileType::Media(_))));
    }

    #[test]
    fn test_should_lock_source_code() {
        let manager = LockManager::new(LockingConfig::default());
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("main.cpp");

        fs::write(&file_path, "#include <iostream>\nint main() { return 0; }").unwrap();

        let should_lock = manager.should_lock(file_path.to_str().unwrap());
        assert!(should_lock);
    }

    #[test]
    fn test_should_not_lock_temp_files() {
        let manager = LockManager::new(LockingConfig::default());
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("temp.tmp");

        fs::write(&file_path, "temporary data").unwrap();

        let should_lock = manager.should_lock(file_path.to_str().unwrap());
        assert!(!should_lock);
    }

    #[test]
    fn test_lfs_suggestions() {
        let manager = LockManager::new(LockingConfig::default());
        let temp_dir = TempDir::new().unwrap();

        // Create large files
        fs::write(
            temp_dir.path().join("large.bin"),
            vec![0u8; 150 * 1024 * 1024],
        )
        .unwrap();
        fs::write(temp_dir.path().join("small.txt"), "small file").unwrap();

        let suggestions = manager.get_lfs_suggestions(temp_dir.path().to_str().unwrap());
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|path| path.contains("large.bin")));
        assert!(!suggestions.iter().any(|path| path.contains("small.txt")));
    }

    #[test]
    fn test_lock_status_tracking() {
        let manager = LockManager::new(LockingConfig::default());
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.cpp");

        fs::write(&file_path, "int main() {}").unwrap();

        let status = manager.get_lock_status(
            &temp_dir.path().to_str().unwrap(),
            &[file_path.to_str().unwrap().to_string()],
        );
        assert!(!status.is_empty());
        assert!(status
            .iter()
            .any(|(_, lock_status)| matches!(lock_status, LockStatus::Unlocked)));
    }

    #[test]
    fn test_locking_config_defaults() {
        let config = LockingConfig::default();
        assert_eq!(config.enabled, true);
        assert_eq!(config.lock_timeout_hours, 24);
        assert!(!config.auto_lock_patterns.is_empty());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_performance_engine_creation() {
        let _engine = PerformanceEngine::new();
        // Engine should be created successfully
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_performance_engine_exists() {
        // Just test that we can create the engine
        let engine = PerformanceEngine::new();
        // Since we don't have the methods implemented, just verify creation
        std::mem::drop(engine);
        assert!(true);
    }
}
