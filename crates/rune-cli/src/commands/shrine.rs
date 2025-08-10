
use anyhow::Result;
use rune_remote::Shrine;
#[derive(clap::Subcommand, Debug)]
pub enum ShrineCmd{ Serve{ #[arg(long, default_value="127.0.0.1:7420")] addr:String } }
pub async fn serve(addr:String)->Result<()> { let addr: std::net::SocketAddr = addr.parse()?; let shrine = Shrine{ root: std::env::current_dir()? }; println!("🕯️  Rune shrine at http://{}", addr); rune_remote::serve(shrine, addr).await }
