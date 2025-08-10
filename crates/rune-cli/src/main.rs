
use clap::{Parser, Subcommand};
mod api;
use api::run_api; use api::serve_api;
use rune_store::Store;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name="rune", version="0.0.1", about="Rune — modern DVCS (0.0.1)")]
struct Args { #[command(subcommand)] cmd: Cmd }

#[derive(Subcommand, Debug)]
enum Cmd {
  /// Run local JSON API server
  Api { #[arg(long, default_value="127.0.0.1:7421")] addr: String,
        #[arg(long)] with_shrine: bool,
        #[arg(long, default_value="127.0.0.1:7420")] shrine_addr: String },
  /// Generate shell completion scripts
  Completions { shell: String },
  /// Generate shell completion scripts
  Completions { shell: String },
  Guide,
  Init,
  Status { #[arg(long, default_value="table")] format: String },
  Add { paths: Vec<std::path::PathBuf> },
  Commit { #[arg(short, long)] message: String },
  Log { #[arg(long, default_value="table")] format: String },
  Branch { name: Option<String>, #[arg(long, default_value="table")] format: String },
  Checkout { name: String },
  Stash { #[arg(long)] apply: bool },
  #[command(subcommand)] Lfs(crate::commands::lfs::LfsCmd),
  #[command(subcommand)] Shrine(crate::commands::shrine::ShrineCmd),
  #[command(subcommand)] Delta(crate::commands::delta::DeltaCmd),
}

fn author() -> rune_core::Author {
  rune_core::Author{ name: whoami::realname(), email: format!("{}@localhost", whoami::username()) }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let a = Args::parse();
  match a.cmd {
    Cmd::Guide => { println!("Rune quick start:\n  rune init\n  rune add <files>\n  rune commit -m \"message\"\n  rune log"); },
    Cmd::Init => { let s = Store::open(std::env::current_dir()?)?; s.create()?; println!("✨ repo initialized"); },
    Cmd::Status { format } => {
      let s = Store::discover(std::env::current_dir()?)?; let idx = s.read_index()?;
      if fmt == "json" { println!("{}", serde_json::json!({"staged": idx.entries.keys().collect::<Vec<_>>()})); }
      else if fmt == "yaml" { println!("{}", serde_yaml::to_string(&serde_json::json!({"staged": idx.entries.keys().collect::<Vec<_>>()}))?); }
      else { println!("Staged: {} files", idx.entries.len()); for k in idx.entries.keys() { println!("  + {}", k); } }
    },
    Cmd::Add { paths } => {
      let s = Store::discover(std::env::current_dir()?)?; if paths.is_empty() { anyhow::bail!("no paths provided"); }
      for p in paths { let rel = p.to_string_lossy().to_string(); s.stage_file(&rel)?; println!("+ {}", rel); }
    },
    Cmd::Commit { message } => {
      let s = Store::discover(std::env::current_dir()?)?; let c = s.commit(&message, author())?; println!("🪄 committed {} — {}", &c.id[..8], c.message);
    },
    Cmd::Log { format } => {
      let s = Store::discover(std::env::current_dir()?)?;
      let list = s.log(); let fmt = format.as_str();
      if fmt == "json" { println!("{}", serde_json::to_string_pretty(&list)?); }
      else if fmt == "yaml" { println!("{}", serde_yaml::to_string(&list)?); }
      else { for c in list.iter().rev() { 
        let ts = chrono::NaiveDateTime::from_timestamp_opt(c.time,0).unwrap();
        println!("{}  {}  [{}]", ts, &c.id[..8], c.message); 
      } }
    },
    Cmd::Branch { name, format } => {
      let s = Store::discover(std::env::current_dir()?)?;
      let fmt = format.as_str(); if let Some(n) = name { std::fs::create_dir_all(s.rune_dir.join("refs/heads"))?; std::fs::write(s.rune_dir.join("refs/heads").join(&n), b"")?; println!("created branch {}", n);
      } else {
        for e in walkdir::WalkDir::new(s.rune_dir.join("refs/heads")) { let e = e?; if e.file_type().is_file() { println!("{}", e.path().file_name().unwrap().to_string_lossy()); } }
      } }
    },
    Cmd::Checkout { name } => {
      let s = Store::discover(std::env::current_dir()?)?; let r = format!("refs/heads/{}", name);
      if s.read_ref(&r).is_none() && !s.rune_dir.join(&r).exists() { anyhow::bail!("branch not found: {}", name); }
      s.set_head(&r)?; println!("switched to {}", name);
    },
    Cmd::Stash { apply } => {
      let s = Store::discover(std::env::current_dir()?)?; let p = s.rune_dir.join("stash.json");
      if apply { if p.exists() { let list: Vec<serde_json::Value> = serde_json::from_slice(&std::fs::read(p.clone())?)?; println!("applied {} stash item(s)", list.len()); std::fs::remove_file(p)?; } else { println!("nothing to apply"); } }
      else { let idx = s.read_index()?; let mut list: Vec<serde_json::Value> = if p.exists() { serde_json::from_slice(&std::fs::read(p.clone())?)? } else { vec![] };
             list.push(serde_json::json!({ "time": chrono::Utc::now().timestamp(), "files": idx.entries.keys().collect::<Vec<_>>() }));
             std::fs::write(p, serde_json::to_vec_pretty(&list)?)?; println!("stashed {} file(s)", idx.entries.len()); s.write_index(&rune_store::Index::default())?; }
    },
    Cmd::Lfs(sub) => crate::commands::lfs::run(sub).await?,
    Cmd::Shrine(sub) => match sub { crate::commands::shrine::ShrineCmd::Serve{addr} => crate::commands::shrine::serve(addr).await? },
    Cmd::Api { addr, with_shrine, shrine_addr } => {
      if with_shrine {
        let api_addr: std::net::SocketAddr = addr.parse()?;
        let shrine_addr: std::net::SocketAddr = shrine_addr.parse()?;
        let shrine = rune_remote::Shrine{ root: std::env::current_dir()? };
        println!("🕯️  Embedded Shrine at http://{}", shrine_addr);
        println!("🔮 Rune API at http://{}", api_addr);
        let s_task = tokio::spawn(async move { rune_remote::serve(shrine, shrine_addr).await });
        let a_task = tokio::spawn(async move { serve_api(api_addr).await });
        let _ = tokio::try_join!(s_task, a_task)?;
      } else {
        run_api(addr).await?;
      }
    },
    Cmd::Delta(sub) => crate::commands::delta::run(sub)?,
    Cmd::Completions { shell } => {
      use clap_complete::{generate, shells::{Bash,Zsh,Fish, PowerShell}};
      let mut cmd = <Args as clap::Parser>::command();
      match shell.as_str() {
        "bash" => generate(Bash, &mut cmd, "rune", &mut std::io::stdout()),
        "zsh" => generate(Zsh, &mut cmd, "rune", &mut std::io::stdout()),
        "fish" => generate(Fish, &mut cmd, "rune", &mut std::io::stdout()),
        "powershell" | "pwsh" => generate(PowerShell, &mut cmd, "rune", &mut std::io::stdout()),
        _ => eprintln!("use: bash|zsh|fish|powershell")
      } }
    },
  }
  Ok(())
}
