use anyhow::Result;
use clap::{Args, Subcommand};
use crate::style::Style;
use rune_planning::{PlanStore, PlanStatus, create_plan, update_status, add_task, add_task_with_meta, update_roots, parse_plan_query, filter_plans, StreamStore, generate_workspace_insights, generate_plan_insights, PLAN_DIR};
use std::env;

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
    /// Show a plan file (optionally with insights)
    Show { id: String, #[arg(long)] insights: bool },
    /// Update status of a plan
    Status { id: String, #[arg(value_enum)] status: PlanStatus },
    /// Add a task line to a plan (appends to Tasks section)
    AddTask { id: String, description: String },
    /// Mark a task (1-based index) done; auto-completes plan if all done
    Done { id: String, task: usize },
    /// Show a lightweight board view of plans and task progress
    Board {
        /// Only include plans with these statuses (comma separated), default all
        #[arg(long)]
        statuses: Option<String>,
        /// Show task lines (not just counts)
        #[arg(long)]
        details: bool,
    },
    /// Filter (slice) plans via simple query string (e.g. "status=active tag=perf root=engine/")
    Slice { query: String },
    /// Add a task with metadata (type, effort, path, tags)
    TaskAdd { id: String, description: String, #[arg(long)] task_type: Option<String>, #[arg(long)] effort: Option<String>, #[arg(long)] path: Option<String>, #[arg(long)] tags: Option<String> },
    /// Set or replace roots for a plan (comma separated)
    SetRoots { id: String, roots: String },
    /// Create a stream (group of plans)
    StreamCreate { title: String, #[arg(long)] tags: Option<String> },
    /// List streams
    StreamList,
    /// Attach plan to stream
    StreamAttach { stream_id: String, plan_id: String },
    /// Generate insights (all plans or one plan if id provided)
    Insights { #[arg(long)] id: Option<String> },
}

// Execute plan related commands using rune-planning crate
pub fn execute_plan_command(args: PlanArgs) -> Result<()> {
    // Root dir is current working directory
    let root = env::current_dir()?;
    let store = PlanStore::new(&root);
    let stream_store = StreamStore::new(&root);
    match args.command {
        PlanCmd::Init => {
            store.ensure()?;
            Style::success("Initialized .rune/plans");
        }
        PlanCmd::Create { title, tags } => {
            store.ensure()?;
            let plan = create_plan(&store, &title, tags.as_deref());
            match plan {
                Ok(p) => Style::success(&format!("Created plan {} -> {}/{}.md", p.id, PLAN_DIR, p.id)),
                Err(e) => Style::error(&format!("Failed creating plan: {e}")),
            }
        }
        PlanCmd::List => {
            store.ensure()?;
            let plans = store.load_all()?;
            if plans.is_empty() { println!("No plans found"); return Ok(()); }
            println!("{:<10} {:<12} {}", "ID", "Status", "Title");
            println!("{}", "-".repeat(60));
            for p in plans { println!("{:<10} {:<12} {}", p.id, p.status.as_str(), p.title); }
        }
        PlanCmd::Show { id, insights } => {
            let plan = store.load(&id)?;
            println!("{}", plan.to_markdown());
            if insights {
                let ins = generate_plan_insights(&plan);
                if !ins.messages.is_empty() {
                    println!("## Insights\n");
                    for m in ins.messages { println!("- {}", m); }
                }
            }
        }
        PlanCmd::Status { id, status } => {
            update_status(&store, &id, status.clone())?;
            Style::success(&format!("Updated {id} status -> {}", status.as_str()));
        }
        PlanCmd::AddTask { id, description } => {
            add_task(&store, &id, &description)?;
            Style::success(&format!("Added task to {id}"));
        }
        PlanCmd::Done { id, task } => {
            match rune_planning::mark_task_done(&store, &id, task)? {
                true => Style::success(&format!("Marked task {task} done in {id}")),
                false => Style::error(&format!("Task index {task} invalid or already done")),
            }
        }
        PlanCmd::Board { statuses, details } => {
            let mut plans = store.load_all()?;
            if let Some(filter) = statuses {
                let wanted: Vec<String> = filter.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
                plans.retain(|p| wanted.iter().any(|w| w == p.status.as_str()));
            }
            if plans.is_empty() { println!("No plans match."); return Ok(()); }
            println!("BOARD ({} plans)\n", plans.len());
            for p in plans {
                let total = p.tasks.len();
                let done = p.tasks.iter().filter(|t| t.done).count();
                let todo = total - done;
                println!("{} [{}] — {} ({} total, {} done, {} todo)", p.id, p.status.as_str(), p.title, total, done, todo);
                if details {
                    if todo > 0 { println!("  Todo:"); }
                    for (i,t) in p.tasks.iter().enumerate() { if !t.done { println!("    {:>2}. {}", i+1, t.description); } }
                    if done > 0 { println!("  Done:"); }
                    for (i,t) in p.tasks.iter().enumerate() { if t.done { println!("    {:>2}. {}", i+1, t.description); } }
                }
                println!();
            }
        }
        PlanCmd::Slice { query } => {
            let plans = store.load_all()?;
            let pq = parse_plan_query(&query);
            let filtered = filter_plans(&plans, &pq);
            if filtered.is_empty() { println!("No plans match slice."); return Ok(()); }
            println!("SLICE ({} plans)", filtered.len());
            for p in filtered { println!("{} [{}] {}", p.id, p.status.as_str(), p.title); }
        }
        PlanCmd::TaskAdd { id, description, task_type, effort, path, tags } => {
            add_task_with_meta(&store, &id, &description, task_type.as_deref(), effort.as_deref(), path.as_deref(), tags.as_deref())?;
            Style::success(&format!("Added task with metadata to {id}"));
        }
        PlanCmd::SetRoots { id, roots } => {
            update_roots(&store, &id, &roots)?;
            Style::success(&format!("Updated roots for {id}"));
        }
        PlanCmd::StreamCreate { title, tags } => {
            let s = stream_store.create(&title, tags.as_deref())?;
            Style::success(&format!("Created stream {}", s.id));
        }
        PlanCmd::StreamList => {
            let streams = stream_store.list()?;
            if streams.is_empty() { println!("No streams"); } else {
                println!("{:<12} {:<20} {:<6} {}", "ID","Title","Plans","Tags");
                for s in streams { println!("{:<12} {:<20} {:<6} {}", s.id, s.title, s.plans.len(), s.tags.join(",")); }
            }
        }
        PlanCmd::StreamAttach { stream_id, plan_id } => {
            stream_store.attach(&stream_id, &plan_id)?;
            Style::success(&format!("Attached {plan_id} to {stream_id}"));
        }
        PlanCmd::Insights { id } => {
            if let Some(plan_id) = id {
                let p = store.load(&plan_id)?;
                let ins = generate_plan_insights(&p);
                println!("Insights for {}:", plan_id);
                for m in ins.messages { println!("- {m}"); }
            } else {
                let plans = store.load_all()?;
                let ws = generate_workspace_insights(&plans);
                println!("Workspace summary:");
                for s in ws.summary { println!("- {s}"); }
                println!("\nPer-plan:");
                for pi in ws.plan_insights { if !pi.messages.is_empty() { println!("{}:", pi.plan_id); for m in pi.messages { println!("  - {m}"); } } }
            }
        }
    }
    Ok(())
}
