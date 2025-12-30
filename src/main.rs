use clap::Parser;
use arc_cli::{run, Args};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    run(&args).await;
    Ok(())
}
