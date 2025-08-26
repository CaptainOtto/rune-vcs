use anyhow::{Result, Context};
use clap::{Args, Subcommand, ValueEnum};
use crate::style::Style;
use std::{fs, path::{Path, PathBuf}};

const PLAN_DIR: &str = ".rune/plans";

#[derive(Debug, Clone, ValueEnum)]
pub enum PlanStatus {
    Planned,
    Active,
    InProgress,
    Blocked,
    Done,
}

impl PlanStatus {
    fn as_str(&self) -> &'static str {
        match self {
            PlanStatus::Planned => "planned",
            PlanStatus::Active => "active",
            PlanStatus::InProgress => "in-progress",
            PlanStatus::Blocked => "blocked",
            PlanStatus::Done => "done",
        }
    }
}

#[derive(Debug, Args)]
pub struct PlanArgs {
    #[command(subcommand)]
    pub command: PlanCmd,
}

#[derive(Debug, Subcommand)]
pub enum PlanCmd {
    /// Initialize planning directory (.rune/plans)
    Init,
    /// Create a new plan file with next id
    Create {
        /// Title of the plan
        title: String,
        /// Optional tags (comma separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// List existing plans (id, status, title)
    List,
    /// Show a plan file
    Show { id: String },
    /// Update status of a plan
    Status { id: String, #[arg(value_enum)] status: PlanStatus },
    /// Add a task line to a plan (appends to Tasks section)
    AddTask { id: String, description: String },
}

fn plan_root() -> PathBuf { PathBuf::from(PLAN_DIR) }

fn ensure_plan_dir() -> Result<()> {
    let dir = plan_root();
    if !dir.exists() { fs::create_dir_all(&dir).with_context(|| format!("create {:?}", dir))?; }
    Ok(())
}

fn next_plan_id() -> Result<String> {
    ensure_plan_dir()?;
    let mut max_num = 0u32;
    for entry in fs::read_dir(plan_root())? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if let Some(stripped) = name.strip_prefix("PLAN-") { if let Some(num_str) = stripped.strip_suffix(".md") { if let Ok(n) = num_str.parse::<u32>() { if n > max_num { max_num = n; } } } }
        }
    }
    Ok(format!("PLAN-{:03}", max_num + 1))
}

fn plan_path(id: &str) -> PathBuf { plan_root().join(format!("{id}.md")) }

fn write_new_plan(id: &str, title: &str, tags: Option<&str>) -> Result<PathBuf> {
    let path = plan_path(id);
    if path.exists() { anyhow::bail!("plan already exists: {id}"); }
    let now = chrono::Utc::now().format("%Y-%m-%d");
    let tags_line = tags.unwrap_or("");
    let content = format!("id: {id}\ntitle: {title}\nstatus: planned\nrelease: \nowners: \ntags: {tags_line}\ncreated: {now}\nupdated: {now}\n\n# Description\n\n(Add details here)\n\n## Goals\n- Example goal\n\n## Tasks\n- [ ] First task\n\n");
    fs::write(&path, content)?;
    Ok(path)
}

fn read_header_lines(path: &Path) -> Result<(String, String, String)> {
    let data = fs::read_to_string(path)?;
    let mut id = String::new();
    let mut status = String::new();
    let mut title = String::new();
    for line in data.lines().take(20) { // header small
        if line.starts_with("id:") { id = line[3..].trim().to_string(); }
        else if line.starts_with("status:") { status = line[7..].trim().to_string(); }
        else if line.starts_with("title:") { title = line[6..].trim().to_string(); }
    }
    Ok((id, status, title))
}

fn update_status(id: &str, new_status: &str) -> Result<()> {
    let path = plan_path(id);
    let mut data = fs::read_to_string(&path).with_context(|| format!("read plan {id}"))?;
    let mut changed = false;
    let mut new_lines = Vec::new();
    for line in data.lines() {
        if line.starts_with("status:") {
            new_lines.push(format!("status: {new_status}"));
            changed = true;
        } else if line.starts_with("updated:") {
            let now = chrono::Utc::now().format("%Y-%m-%d");
            new_lines.push(format!("updated: {now}"));
        } else {
            new_lines.push(line.to_string());
        }
    }
    if changed { fs::write(&path, new_lines.join("\n") + "\n")?; }
    Ok(())
}

fn append_task(id: &str, desc: &str) -> Result<()> {
    let path = plan_path(id);
    let mut data = fs::read_to_string(&path)?;
    // Find Tasks section marker or append
    if let Some(idx) = data.find("## Tasks") {
        // insert after last line
        data.push_str(&format!("- [ ] {desc}\n"));
    } else {
        data.push_str("\n## Tasks\n");
        data.push_str(&format!("- [ ] {desc}\n"));
    }
    // update updated date
    let mut lines: Vec<String> = data.lines().map(|s| s.to_string()).collect();
    for line in lines.iter_mut() { if line.starts_with("updated:") { *line = format!("updated: {}", chrono::Utc::now().format("%Y-%m-%d")); } }
    fs::write(&path, lines.join("\n") + "\n")?;
    Ok(())
}

pub fn execute_plan_command(args: PlanArgs) -> Result<()> {
    match args.command {
        PlanCmd::Init => {
            ensure_plan_dir()?;
            Style::success("Initialized .rune/plans");
        }
        PlanCmd::Create { title, tags } => {
            ensure_plan_dir()?;
            let id = next_plan_id()?;
            let path = write_new_plan(&id, &title, tags.as_deref())?;
            Style::success(&format!("Created plan {id} -> {}", path.display()));
        }
        PlanCmd::List => {
            ensure_plan_dir()?;
            let mut rows = Vec::new();
            for entry in fs::read_dir(plan_root())? { let entry = entry?; if entry.path().extension().and_then(|s| s.to_str()) == Some("md") { let (id, status, title) = read_header_lines(&entry.path())?; if !id.is_empty() { rows.push((id, status, title)); } } }
            if rows.is_empty() { println!("No plans found"); return Ok(()); }
            println!("{:<10} {:<12} {}", "ID", "Status", "Title");
            println!("{}", "-".repeat(60));
            rows.sort_by(|a,b| a.0.cmp(&b.0));
            for (id, status, title) in rows { println!("{:<10} {:<12} {}", id, status, title); }
        }
        PlanCmd::Show { id } => {
            let path = plan_path(&id);
            let data = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
            println!("{}", data);
        }
        PlanCmd::Status { id, status } => {
            update_status(&id, status.as_str())?;
            Style::success(&format!("Updated {id} status -> {}", status.as_str()));
        }
        PlanCmd::AddTask { id, description } => {
            append_task(&id, &description)?;
            Style::success(&format!("Added task to {id}"));
        }
    }
    Ok(())
}
