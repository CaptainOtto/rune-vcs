
use anyhow::Result;
use clap::Subcommand;
use std::io::Read;

#[derive(Subcommand, Debug)]
pub enum DeltaCmd{
  Make{ base: std::path::PathBuf, new: std::path::PathBuf, #[arg(short,long)] out: std::path::PathBuf, #[arg(short,long, default_value_t=64usize)] chunk: usize },
  Apply{ base: std::path::PathBuf, patch: std::path::PathBuf, #[arg(short,long)] out: std::path::PathBuf },
}

pub fn run(cmd:DeltaCmd)->Result<()>{
    match cmd{
        DeltaCmd::Make{base,new,out,chunk} => {
            let b = std::fs::read(base)?; let n = std::fs::read(new)?;
            let p = rune_delta::make(&b, &n, chunk)?;
            std::fs::write(out, serde_json::to_vec_pretty(&p)?)?;
            println!("delta written");
        }
        DeltaCmd::Apply{base,patch,out} => {
            let b = std::fs::read(base)?; let p: rune_delta::Patch = serde_json::from_slice(&std::fs::read(patch)?)?;
            let r = rune_delta::apply(&b, &p)?; if let Some(pp)=out.parent(){ std::fs::create_dir_all(pp)?; } std::fs::write(out, r)?;
            println!("applied");
        }
    }
    Ok(())
}
