use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use clap::ValueEnum;
use std::{fs, path::PathBuf};
use std::io::Write;

pub const PLAN_DIR: &str = ".rune/plans";
pub const CONFIG_FILE: &str = ".rune/planning.toml";
pub const STREAM_DIR: &str = ".rune/streams";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum PlanStatus { Planned, Active, InProgress, Blocked, Done }

impl PlanStatus { pub fn as_str(&self) -> &'static str { match self { Self::Planned=>"planned", Self::Active=>"active", Self::InProgress=>"in-progress", Self::Blocked=>"blocked", Self::Done=>"done" } } }
impl std::fmt::Display for PlanStatus { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.as_str()) } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub description: String,
    pub done: bool,
    pub task_type: Option<String>,
    pub effort: Option<String>,
    pub path: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub status: PlanStatus,
    pub release: Option<String>,
    pub owners: Vec<String>,
    pub tags: Vec<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub goals: Vec<String>,
    pub tasks: Vec<Task>,
    pub roots: Vec<String>,
    pub description: String,
}

impl Plan {
    pub fn to_markdown(&self) -> String {
        let tags = self.tags.join(",");
        let owners = self.owners.join(",");
        let goals_md = if self.goals.is_empty() { "".into() } else { self.goals.iter().map(|g| format!("- {g}")).collect::<Vec<_>>().join("\n") };
        let tasks_md = self.tasks.iter().map(|t| {
            let mut meta_parts = Vec::new();
            if let Some(ref ty) = t.task_type { meta_parts.push(format!("type:{}", ty)); }
            if let Some(ref e) = t.effort { meta_parts.push(format!("effort:{}", e)); }
            if let Some(ref p) = t.path { meta_parts.push(format!("path:{}", p)); }
            if !t.tags.is_empty() { meta_parts.push(format!("tags:{}", t.tags.join("|"))); }
            let meta = if meta_parts.is_empty() { String::new() } else { format!(" {{{}}}", meta_parts.join(" ")) };
            format!("- [{}] {}{}", if t.done {"x"} else {" "}, t.description, meta)
        }).collect::<Vec<_>>().join("\n");
    let roots = if self.roots.is_empty() { String::new() } else { self.roots.join(",") };
    format!("id: {id}\ntitle: {title}\nstatus: {status}\nrelease: {release}\nowners: {owners}\ntags: {tags}\nroots: {roots}\ncreated: {created}\nupdated: {updated}\n\n# Description\n\n{desc}\n\n## Goals\n{goals}\n\n## Tasks\n{tasks}\n", id=self.id, title=self.title, status=self.status, release=self.release.clone().unwrap_or_default(), owners=owners, tags=tags, roots=roots, created=self.created.format("%Y-%m-%d"), updated=self.updated.format("%Y-%m-%d"), desc=self.description, goals=goals_md, tasks=tasks_md)
    }

    pub fn parse_markdown(md: &str) -> Result<Self> {
        let mut id = String::new();
        let mut title = String::new();
        let mut status = PlanStatus::Planned;
        let mut release = None;
        let mut owners: Vec<String> = vec![];
        let mut tags: Vec<String> = vec![];
        let mut created: Option<DateTime<Utc>> = None;
        let mut updated: Option<DateTime<Utc>> = None;
        let mut description_lines = Vec::new();
        let mut in_description = false;
        let mut goals: Vec<String> = Vec::new();
        let mut tasks: Vec<Task> = Vec::new();
        let mut section = "";
        let mut roots: Vec<String> = Vec::new();
        for line in md.lines() {
            if line.starts_with("id:") { id = line[3..].trim().to_string(); }
            else if line.starts_with("title:") { title = line[6..].trim().to_string(); }
            else if line.starts_with("status:") { let v = line[7..].trim(); status = match v {"planned"=>PlanStatus::Planned,"active"=>PlanStatus::Active,"in-progress"=>PlanStatus::InProgress,"blocked"=>PlanStatus::Blocked,"done"=>PlanStatus::Done,_=>PlanStatus::Planned}; }
            else if line.starts_with("release:") { let v = line[8..].trim(); if !v.is_empty() { release = Some(v.to_string()); } }
            else if line.starts_with("owners:") { owners = line[7..].trim().split(',').filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect(); }
            else if line.starts_with("tags:") { tags = line[5..].trim().split(',').filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect(); }
            else if line.starts_with("roots:") { roots = line[6..].trim().split(',').filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect(); }
            else if line.starts_with("created:") { let d = line[8..].trim(); created = Some(parse_date(d)?); }
            else if line.starts_with("updated:") { let d = line[8..].trim(); updated = Some(parse_date(d)?); }
            else if line.starts_with("# Description") { section = "description"; in_description = true; }
            else if line.starts_with("## Goals") { section = "goals"; in_description = false; }
            else if line.starts_with("## Tasks") { section = "tasks"; in_description = false; }
            else {
                match section {
                    "description" => { if in_description { description_lines.push(line.to_string()); } },
                    "goals" => { if line.trim_start().starts_with('-') { goals.push(line.trim_start().trim_start_matches('-').trim().to_string()); } },
                    "tasks" => { if line.trim_start().starts_with('-') { let rest = line.trim_start().trim_start_matches('-').trim(); let done = rest.starts_with("[x]") || rest.starts_with("[X]"); let mut body = rest; if done { body = body.trim_start_matches("[x]").trim_start_matches("[X]").trim(); } else if body.starts_with("[ ]") { body = body.trim_start_matches("[ ]").trim(); }
                        // Extract metadata block {...}
                        let (desc_part, meta_part) = if let Some(idx) = body.rfind('{') { if body.ends_with('}') { (body[..idx].trim(), Some(&body[idx+1..body.len()-1])) } else { (body, None) } } else { (body, None) };
                        let mut task_type=None; let mut effort=None; let mut path=None; let mut ttags=Vec::new();
                        if let Some(meta) = meta_part { for token in meta.split_whitespace() { if let Some((k,v)) = token.split_once(':') { match k { "type"=>task_type=Some(v.to_string()), "effort"=>effort=Some(v.to_string()), "path"=>path=Some(v.to_string()), "tags"=>{ ttags = v.split('|').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect(); }, _=>{} } } } }
                        tasks.push(Task { description: desc_part.to_string(), done, task_type, effort, path, tags: ttags }); } },
                    _ => {}
                }
            }
        }
        Ok(Self { id, title, status, release, owners, tags, created: created.unwrap_or_else(Utc::now), updated: updated.unwrap_or_else(Utc::now), goals, tasks, roots, description: description_lines.join("\n").trim().to_string() })
    }
}

fn parse_date(d: &str) -> Result<DateTime<Utc>> {
    let naive = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")?.and_hms_opt(0, 0, 0).unwrap();
    Ok(DateTime::from_naive_utc_and_offset(naive, Utc))
}

pub struct PlanStore { root: PathBuf }
impl PlanStore {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    fn dir(&self) -> PathBuf { self.root.join(PLAN_DIR) }
    pub fn ensure(&self) -> Result<()> { if !self.dir().exists() { fs::create_dir_all(self.dir())?; } Ok(()) }
    pub fn next_id(&self) -> Result<String> { self.ensure()?; let mut max = 0u32; for e in fs::read_dir(self.dir())? { let e = e?; if let Some(name) = e.file_name().to_str() { if let Some(rest) = name.strip_prefix("PLAN-") { if let Some(num) = rest.strip_suffix(".md") { if let Ok(n) = num.parse::<u32>() { if n>max { max=n; } } } } } } Ok(format!("PLAN-{:03}", max+1)) }
    pub fn load_all(&self) -> Result<Vec<Plan>> { self.ensure()?; let mut v = Vec::new(); if self.dir().exists() { for e in fs::read_dir(self.dir())? { let e = e?; if e.path().extension().and_then(|s| s.to_str()) == Some("md") { let text = fs::read_to_string(e.path())?; if let Ok(p) = Plan::parse_markdown(&text) { v.push(p); } } } } v.sort_by(|a,b| a.id.cmp(&b.id)); Ok(v) }
    pub fn path_for(&self, id: &str) -> PathBuf { self.dir().join(format!("{id}.md")) }
    pub fn save(&self, plan: &Plan) -> Result<()> { self.ensure()?; fs::write(self.path_for(&plan.id), plan.to_markdown())?; Ok(()) }
    pub fn load(&self, id: &str) -> Result<Plan> { let text = fs::read_to_string(self.path_for(id)).with_context(|| format!("load plan {id}"))?; Plan::parse_markdown(&text) }
}

pub fn create_plan(store: &PlanStore, title: &str, tags: Option<&str>) -> Result<Plan> {
    let id = store.next_id()?;
    let now = Utc::now();
    let p = Plan { id: id.clone(), title: title.to_string(), status: PlanStatus::Planned, release: None, owners: vec![], tags: tags.unwrap_or("").split(',').filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect(), created: now, updated: now, goals: vec![], tasks: vec![Task { description: "First task".into(), done: false, task_type: None, effort: None, path: None, tags: vec![] }], roots: vec![], description: "(Add details here)".into() };
    store.save(&p)?; Ok(p)
}

fn log_signal(root: &PathBuf, kind: &str, kv: &[(&str, &str)]) -> Result<()> {
    let dir = root.join(".rune/index");
    fs::create_dir_all(&dir)?;
    let path = dir.join("signals.log");
    let ts = Utc::now().to_rfc3339();
    let mut line = format!("{ts} kind={kind}");
    for (k,v) in kv { line.push(' '); line.push_str(k); line.push('='); line.push_str(v); }
    line.push('\n');
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(line.as_bytes())?;
    Ok(())
}

// ---- Streams ----
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub plans: Vec<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub description: String,
}

impl Stream {
    pub fn to_markdown(&self) -> String {
        format!("id: {id}\ntitle: {title}\ntags: {tags}\nplans: {plans}\ncreated: {created}\nupdated: {updated}\n\n# Description\n\n{desc}\n", id=self.id, title=self.title, tags=self.tags.join(","), plans=self.plans.join(","), created=self.created.format("%Y-%m-%d"), updated=self.updated.format("%Y-%m-%d"), desc=self.description)
    }
    pub fn parse(md: &str) -> Result<Self> {
        let mut id=String::new(); let mut title=String::new(); let mut tags=Vec::new(); let mut plans=Vec::new(); let mut created=None; let mut updated=None; let mut desc_lines=Vec::new(); let mut in_desc=false;
        for line in md.lines() {
            if line.starts_with("id:") { id=line[3..].trim().to_string(); }
            else if line.starts_with("title:") { title=line[6..].trim().to_string(); }
            else if line.starts_with("tags:") { tags=line[5..].trim().split(',').filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect(); }
            else if line.starts_with("plans:") { plans=line[6..].trim().split(',').filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect(); }
            else if line.starts_with("created:") { created=Some(parse_date(&line[8..].trim())?); }
            else if line.starts_with("updated:") { updated=Some(parse_date(&line[8..].trim())?); }
            else if line.starts_with("# Description") { in_desc=true; }
            else if in_desc { desc_lines.push(line.to_string()); }
        }
        Ok(Stream { id, title, tags, plans, created: created.unwrap_or_else(Utc::now), updated: updated.unwrap_or_else(Utc::now), description: desc_lines.join("\n").trim().to_string() })
    }
}

pub struct StreamStore { root: PathBuf }
impl StreamStore {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    fn dir(&self) -> PathBuf { self.root.join(STREAM_DIR) }
    fn ensure(&self) -> Result<()> { if !self.dir().exists() { fs::create_dir_all(self.dir())?; } Ok(()) }
    fn next_id(&self) -> Result<String> { self.ensure()?; let mut max=0u32; for e in fs::read_dir(self.dir())? { let e=e?; if let Some(name)=e.file_name().to_str() { if let Some(rest)=name.strip_prefix("STREAM-") { if let Some(num)=rest.strip_suffix(".md") { if let Ok(n)=num.parse::<u32>() { if n>max { max=n; } } } } } } Ok(format!("STREAM-{:03}", max+1)) }
    pub fn create(&self, title:&str, tags: Option<&str>) -> Result<Stream> { let id=self.next_id()?; let now=Utc::now(); let s=Stream { id: id.clone(), title: title.to_string(), tags: tags.unwrap_or("").split(',').filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect(), plans: vec![], created: now, updated: now, description: "(Add details)".into() }; self.save(&s)?; log_signal(&self.root, "stream_created", &[ ("stream", &id) ])?; Ok(s) }
    pub fn path_for(&self, id:&str) -> PathBuf { self.dir().join(format!("{id}.md")) }
    pub fn save(&self, s:&Stream) -> Result<()> { self.ensure()?; fs::write(self.path_for(&s.id), s.to_markdown())?; Ok(()) }
    pub fn load(&self, id:&str) -> Result<Stream> { let text=fs::read_to_string(self.path_for(id))?; Stream::parse(&text) }
    pub fn list(&self) -> Result<Vec<Stream>> { self.ensure()?; let mut v=Vec::new(); for e in fs::read_dir(self.dir())? { let e=e?; if e.path().extension().and_then(|s| s.to_str())==Some("md") { if let Ok(st)=Stream::parse(&fs::read_to_string(e.path())?) { v.push(st); } } } v.sort_by(|a,b| a.id.cmp(&b.id)); Ok(v) }
    pub fn attach(&self, stream_id:&str, plan_id:&str) -> Result<()> { let mut s=self.load(stream_id)?; if !s.plans.contains(&plan_id.to_string()) { s.plans.push(plan_id.to_string()); s.updated=Utc::now(); self.save(&s)?; log_signal(&self.root, "stream_attach", &[ ("stream", stream_id), ("plan", plan_id) ])?; } Ok(()) }
}

// --- Query / Slice filtering ---
#[derive(Debug, Default)]
pub struct PlanQuery {
    pub statuses: Vec<String>,
    pub tags: Vec<String>,
    pub roots: Vec<String>,
    pub text: Option<String>,
    pub path: Option<String>,
}

pub fn parse_plan_query(q:&str) -> PlanQuery {
    let mut pq=PlanQuery::default();
    for tok in q.split_whitespace() {
        if let Some((k,v))=tok.split_once('=') {
            match k {
                "status"|"statuses" => pq.statuses.extend(v.split(',').map(|s| s.to_lowercase())),
                "tag"|"tags" => pq.tags.extend(v.split(',').map(|s| s.to_lowercase())),
                "root"|"roots" => pq.roots.extend(v.split(',').map(|s| s.to_string())),
                "path" => pq.path = Some(v.to_string()),
                "text"|"q" => pq.text = Some(v.to_string()),
                _ => {}
            }
        }
    }
    pq
}

pub fn filter_plans(plans: &[Plan], query:&PlanQuery) -> Vec<Plan> {
    plans.iter().filter(|p| {
        if !query.statuses.is_empty() && !query.statuses.iter().any(|s| s==p.status.as_str()) { return false; }
        if !query.tags.is_empty() && query.tags.iter().any(|t| !p.tags.iter().any(|pt| pt.to_lowercase()==*t)) { return false; }
        if !query.roots.is_empty() && !p.roots.iter().any(|r| query.roots.iter().any(|qr| r.starts_with(qr))) { return false; }
        if let Some(ref txt)=query.text { let t=txt.to_lowercase(); if !p.title.to_lowercase().contains(&t) && !p.description.to_lowercase().contains(&t) { return false; } }
        if let Some(ref path)=query.path { if !p.tasks.iter().any(|t| t.path.as_deref().map(|pp| pp.starts_with(path)).unwrap_or(false)) { return false; } }
        true
    }).cloned().collect()
}

// ---- Insights (lightweight heuristic, AI-ready stub) ----
pub struct PlanInsight { pub plan_id: String, pub messages: Vec<String> }

pub fn generate_plan_insights(plan: &Plan) -> PlanInsight {
    let mut msgs = Vec::new();
    let total = plan.tasks.len();
    let done = plan.tasks.iter().filter(|t| t.done).count();
    if total>0 { msgs.push(format!("Progress: {done}/{total} tasks ({:.0}%)", (done as f32/ total as f32)*100.0)); }
    if plan.goals.is_empty() { msgs.push("No goals defined; consider adding 2–5 high-level goals.".into()); }
    let missing_effort = plan.tasks.iter().filter(|t| !t.done && t.effort.is_none()).count();
    if missing_effort > 0 { msgs.push(format!("{} open tasks lack effort sizing.", missing_effort)); }
    let long_titles = plan.tasks.iter().filter(|t| !t.done && t.description.split_whitespace().count()>18).count();
    if long_titles>0 { msgs.push(format!("{} tasks look verbose—may benefit from splitting.", long_titles)); }
    if plan.roots.is_empty() { msgs.push("No roots set; add code roots to enable path-focused slices.".into()); }
    let typed = plan.tasks.iter().filter(|t| t.task_type.is_some()).count();
    if typed < total && total>0 { msgs.push(format!("Only {}/{} tasks have a type; add types for better analytics.", typed, total)); }
    PlanInsight { plan_id: plan.id.clone(), messages: msgs }
}

pub struct WorkspaceInsights { pub plan_insights: Vec<PlanInsight>, pub summary: Vec<String> }

pub fn generate_workspace_insights(plans: &[Plan]) -> WorkspaceInsights {
    let mut plan_insights = Vec::new();
    for p in plans { plan_insights.push(generate_plan_insights(p)); }
    // Aggregate
    let total_plans = plans.len();
    let active = plans.iter().filter(|p| matches!(p.status, PlanStatus::Active|PlanStatus::InProgress)).count();
    let blocked = plans.iter().filter(|p| matches!(p.status, PlanStatus::Blocked)).count();
    let mut summary = vec![format!("Plans: {} (active {}, blocked {})", total_plans, active, blocked)];
    let avg_completion: f32 = if total_plans>0 { plans.iter().map(|p| if p.tasks.is_empty(){0.0}else{ p.tasks.iter().filter(|t| t.done).count() as f32 / p.tasks.len() as f32 }).sum::<f32>() / total_plans as f32 } else {0.0};
    summary.push(format!("Avg task completion {:.0}%", avg_completion*100.0));
    WorkspaceInsights { plan_insights, summary }
}

pub fn update_status(store: &PlanStore, id: &str, status: PlanStatus) -> Result<()> { let mut p = store.load(id)?; p.status = status; p.updated = Utc::now(); store.save(&p)?; log_signal(&store.root, "status_change", &[ ("plan", &p.id), ("status", p.status.as_str()) ])?; Ok(()) }
pub fn add_task(store: &PlanStore, id: &str, desc: &str) -> Result<()> { let mut p = store.load(id)?; p.tasks.push(Task { description: desc.into(), done: false, task_type: None, effort: None, path: None, tags: vec![] }); p.updated = Utc::now(); store.save(&p)?; log_signal(&store.root, "task_added", &[ ("plan", &p.id), ("count", &p.tasks.len().to_string()) ])?; Ok(()) }
pub fn add_task_with_meta(store: &PlanStore, id: &str, desc: &str, task_type: Option<&str>, effort: Option<&str>, path: Option<&str>, tags: Option<&str>) -> Result<()> {
    let mut p = store.load(id)?;
    let tag_list = tags.unwrap_or("").split(',').filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect();
    p.tasks.push(Task { description: desc.into(), done: false, task_type: task_type.map(|s| s.to_string()), effort: effort.map(|s| s.to_string()), path: path.map(|s| s.to_string()), tags: tag_list });
    p.updated = Utc::now();
    store.save(&p)?;
    log_signal(&store.root, "task_added", &[ ("plan", &p.id), ("count", &p.tasks.len().to_string()) ])?;
    Ok(())
}
pub fn update_roots(store: &PlanStore, id: &str, roots: &str) -> Result<()> {
    let mut p = store.load(id)?;
    p.roots = roots.split(',').filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect();
    p.updated = Utc::now();
    store.save(&p)?;
    log_signal(&store.root, "roots_set", &[ ("plan", &p.id), ("roots_count", &p.roots.len().to_string()) ])?;
    Ok(())
}
pub fn mark_task_done(store: &PlanStore, id: &str, index_one_based: usize) -> Result<bool> {
    let mut p = store.load(id)?;
    if index_one_based == 0 || index_one_based > p.tasks.len() { return Ok(false); }
    let idx = index_one_based - 1;
    if !p.tasks[idx].done { p.tasks[idx].done = true; p.updated = Utc::now();
        let all_done = !p.tasks.is_empty() && p.tasks.iter().all(|t| t.done);
        if all_done { p.status = PlanStatus::Done; }
        store.save(&p)?;
        log_signal(&store.root, "task_done", &[ ("plan", &p.id), ("task_index", &index_one_based.to_string()), ("all_done", &all_done.to_string()) ])?;
        return Ok(true);
    }
    Ok(false)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningConfig {
    #[serde(default = "default_auto_complete")] pub auto_complete_on_all_tasks_done: bool,
    #[serde(default)] pub archive_done_after_days: Option<u32>,
    #[serde(default)] pub board_default_status_filters: Option<Vec<String>>,
}

fn default_auto_complete() -> bool { true }

impl Default for PlanningConfig { fn default() -> Self { Self { auto_complete_on_all_tasks_done: true, archive_done_after_days: None, board_default_status_filters: None } } }

impl PlanningConfig {
    pub fn load(root: &PathBuf) -> Result<Self> {
        let path = root.join(CONFIG_FILE);
        if !path.exists() { return Ok(Self::default()); }
        let data = fs::read_to_string(path)?;
        Ok(toml::from_str(&data).unwrap_or_default())
    }
    pub fn save(&self, root: &PathBuf) -> Result<()> {
        let path = root.join(CONFIG_FILE);
        if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn create_and_load_plan() -> Result<()> {
        let tmp = TempDir::new().unwrap();
        let store = PlanStore::new(tmp.path());
        store.ensure()?;
        let p = create_plan(&store, "Test Plan", Some("alpha,beta"))?;
        assert!(p.id.starts_with("PLAN-"));
        let all = store.load_all()?;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Test Plan");
        Ok(())
    }

    #[test]
    fn status_and_task_updates() -> Result<()> {
        let tmp = TempDir::new().unwrap();
        let store = PlanStore::new(tmp.path());
        let p = create_plan(&store, "Work", None)?;
        update_status(&store, &p.id, PlanStatus::Active)?;
        add_task(&store, &p.id, "Do something")?;
        let loaded = store.load(&p.id)?;
        assert_eq!(loaded.status, PlanStatus::Active);
        assert!(loaded.tasks.iter().any(|t| t.description == "Do something"));
        Ok(())
    }

    #[test]
    fn mark_task_done_and_auto_complete() -> Result<()> {
        let tmp = TempDir::new().unwrap();
        let store = PlanStore::new(tmp.path());
        let p = create_plan(&store, "Auto", None)?;
        // initial first task incomplete
        assert_eq!(store.load(&p.id)?.tasks[0].done, false);
        assert!(mark_task_done(&store, &p.id, 1)?);
        let after = store.load(&p.id)?;
        assert!(after.tasks[0].done);
        // All tasks done -> plan status done
        assert_eq!(after.status, PlanStatus::Done);
        Ok(())
    }
}
