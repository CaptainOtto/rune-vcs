use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum DeltaCmd {
    Make {
        base: std::path::PathBuf,
        new: std::path::PathBuf,
        #[arg(short, long)]
        out: std::path::PathBuf,
        #[arg(short, long, default_value_t = 64usize)]
        chunk: usize,
    },
    Apply {
        base: std::path::PathBuf,
        patch: std::path::PathBuf,
        #[arg(short, long)]
        out: std::path::PathBuf,
    },
    /// Enhanced diff with rename/copy detection
    Diff {
        #[arg(help = "Old file or directory")]
        old: std::path::PathBuf,
        #[arg(help = "New file or directory")]
        new: std::path::PathBuf,
        #[arg(long, help = "Diff mode: character, word, or line", default_value = "line")]
        mode: String,
        #[arg(long, help = "Detect file renames")]
        detect_renames: bool,
        #[arg(long, help = "Detect file copies")]
        detect_copies: bool,
        #[arg(long, help = "Similarity threshold for rename/copy detection", default_value = "0.7")]
        similarity: f64,
        #[arg(long, help = "Context lines to show", default_value = "3")]
        context: usize,
    },
    /// Calculate similarity between two files
    Similarity {
        #[arg(help = "First file")]
        file1: std::path::PathBuf,
        #[arg(help = "Second file")]
        file2: std::path::PathBuf,
    },
}

pub fn run(cmd: DeltaCmd) -> Result<()> {
    match cmd {
        DeltaCmd::Make {
            base,
            new,
            out,
            chunk,
        } => {
            let b = std::fs::read(base)?;
            let n = std::fs::read(new)?;
            let p = rune_delta::make(&b, &n, chunk)?;
            std::fs::write(out, serde_json::to_vec_pretty(&p)?)?;
            println!("delta written");
        }
        DeltaCmd::Apply { base, patch, out } => {
            let b = std::fs::read(base)?;
            let p: rune_delta::Patch = serde_json::from_slice(&std::fs::read(patch)?)?;
            let r = rune_delta::apply(&b, &p)?;
            if let Some(pp) = out.parent() {
                std::fs::create_dir_all(pp)?;
            }
            std::fs::write(out, r)?;
            println!("applied");
        }
        DeltaCmd::Diff { old, new, mode, detect_renames, detect_copies, similarity, context } => {
            let diff_mode = match mode.to_lowercase().as_str() {
                "character" | "char" => rune_delta::DiffMode::Character,
                "word" => rune_delta::DiffMode::Word,
                "line" => rune_delta::DiffMode::Line,
                _ => anyhow::bail!("Invalid diff mode. Use: character, word, or line"),
            };

            let options = rune_delta::DiffOptions {
                mode: diff_mode,
                detect_renames,
                detect_copies,
                similarity_threshold: similarity,
                context_lines: context,
            };

            if old.is_file() && new.is_file() {
                // Single file diff
                let old_content = std::fs::read(&old)?;
                let new_content = std::fs::read(&new)?;
                
                let diff_result = rune_delta::enhanced_diff(&old_content, &new_content, &options)?;
                println!("📄 Diff: {} -> {}", old.display(), new.display());
                println!("{}", diff_result);
            } else if old.is_dir() && new.is_dir() {
                // Directory diff with rename/copy detection
                use std::collections::HashMap;
                
                println!("📁 Directory diff: {} -> {}", old.display(), new.display());
                
                let mut old_files = HashMap::new();
                let mut new_files = HashMap::new();
                
                // Collect files from old directory
                for entry in walkdir::WalkDir::new(&old) {
                    let entry = entry?;
                    if entry.file_type().is_file() {
                        let rel_path = entry.path().strip_prefix(&old)?.to_string_lossy().to_string();
                        let content = std::fs::read(entry.path())?;
                        old_files.insert(rel_path, content);
                    }
                }
                
                // Collect files from new directory
                for entry in walkdir::WalkDir::new(&new) {
                    let entry = entry?;
                    if entry.file_type().is_file() {
                        let rel_path = entry.path().strip_prefix(&new)?.to_string_lossy().to_string();
                        let content = std::fs::read(entry.path())?;
                        new_files.insert(rel_path, content);
                    }
                }
                
                // Find deleted and added files
                let mut deleted_files = HashMap::new();
                let mut added_files = HashMap::new();
                
                for (path, content) in &old_files {
                    if !new_files.contains_key(path) {
                        deleted_files.insert(path.clone(), content.clone());
                    }
                }
                
                for (path, content) in &new_files {
                    if !old_files.contains_key(path) {
                        added_files.insert(path.clone(), content.clone());
                    }
                }
                
                // Detect renames if enabled
                if options.detect_renames {
                    let renames = rune_delta::detect_renames(&deleted_files, &added_files, options.similarity_threshold);
                    for rename in renames {
                        println!("🔄 Rename: {} -> {} (similarity: {:.2})", 
                            rename.old_path, rename.new_path, rename.similarity);
                    }
                }
                
                // Detect copies if enabled
                if options.detect_copies {
                    let mut existing_files = HashMap::new();
                    for (path, content) in &old_files {
                        if new_files.contains_key(path) {
                            existing_files.insert(path.clone(), content.clone());
                        }
                    }
                    
                    let copies = rune_delta::detect_copies(&existing_files, &added_files, options.similarity_threshold);
                    for copy in copies {
                        println!("📋 Copy: {} -> {} (similarity: {:.2})", 
                            copy.source_path, copy.dest_path, copy.similarity);
                    }
                }
                
                // Show modified files
                for (path, old_content) in &old_files {
                    if let Some(new_content) = new_files.get(path) {
                        if old_content != new_content {
                            println!("\n📝 Modified: {}", path);
                            let file_diff = rune_delta::enhanced_diff(old_content, new_content, &options)?;
                            println!("{}", file_diff);
                        }
                    }
                }
            } else {
                anyhow::bail!("Both paths must be either files or directories");
            }
        }
        DeltaCmd::Similarity { file1, file2 } => {
            let content1 = std::fs::read(&file1)?;
            let content2 = std::fs::read(&file2)?;
            
            let similarity = rune_delta::calculate_similarity(&content1, &content2);
            
            println!("📊 Similarity between {} and {}:", file1.display(), file2.display());
            println!("  Similarity score: {:.4} ({:.1}%)", similarity, similarity * 100.0);
            
            if similarity > 0.8 {
                println!("  ✅ Very similar files");
            } else if similarity > 0.5 {
                println!("  🟡 Moderately similar files");
            } else {
                println!("  🔴 Different files");
            }
        }
    }
    Ok(())
}
