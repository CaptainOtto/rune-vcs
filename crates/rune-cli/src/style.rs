use colored::*;
use console::Term;

pub struct Style;

impl Style {
    /// Print a success message
    pub fn success(msg: &str) {
        println!("{} {}", "✓".green().bold(), msg);
    }

    /// Print an info message
    pub fn info(msg: &str) {
        println!("{} {}", "ℹ".blue().bold(), msg);
    }

    /// Print a warning message
    pub fn warning(msg: &str) {
        println!("{} {}", "⚠".yellow().bold(), msg);
    }

    /// Print an error message
    pub fn error(msg: &str) {
        eprintln!("{} {}", "✗".red().bold(), msg);
    }

    /// Print a commit hash with styling
    pub fn commit_hash(hash: &str) -> ColoredString {
        hash.yellow().bold()
    }

    /// Print a branch name with styling
    pub fn branch_name(name: &str) -> ColoredString {
        name.green().bold()
    }

    /// Print a file path with styling
    pub fn file_path(path: &str) -> ColoredString {
        path.cyan()
    }

    /// Print a status indicator
    pub fn status_added() -> ColoredString {
        "+".green().bold()
    }

    pub fn status_modified() -> ColoredString {
        "M".yellow().bold()
    }

    pub fn status_deleted() -> ColoredString {
        "-".red().bold()
    }

    pub fn status_renamed() -> ColoredString {
        "R".blue().bold()
    }

    pub fn status_untracked() -> ColoredString {
        "?".magenta().bold()
    }

    /// Format a timestamp nicely
    pub fn timestamp(ts: chrono::NaiveDateTime) -> String {
        ts.format("%Y-%m-%d %H:%M:%S").to_string().dimmed().to_string()
    }

    /// Print a section header
    pub fn section_header(title: &str) {
        println!("\n{}", title.bold().underline());
    }

    /// Print a table-like row
    pub fn table_row(col1: &str, col2: &str, col3: &str) {
        println!(
            "{:<12} {:<10} {}",
            col1.yellow().bold(),
            col2.dimmed(),
            col3
        );
    }

    /// Print a summary line
    pub fn summary(items: usize, item_type: &str) {
        if items == 0 {
            println!("{}", format!("No {} found", item_type).dimmed());
        } else {
            println!(
                "{} {}{}",
                items.to_string().bold(),
                item_type,
                if items == 1 { "" } else { "s" }
            );
        }
    }

    /// Check if we should use colors (respects NO_COLOR env var)
    pub fn should_color() -> bool {
        std::env::var("NO_COLOR").is_err() && Term::stdout().features().colors_supported()
    }
}

/// Initialize colored output
pub fn init_colors() {
    colored::control::set_override(Style::should_color());
}

/// Format file size nicely
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration nicely
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{} seconds ago", seconds)
    } else if seconds < 3600 {
        format!("{} minutes ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours ago", seconds / 3600)
    } else {
        format!("{} days ago", seconds / 86400)
    }
}
