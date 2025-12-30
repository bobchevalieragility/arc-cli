use clap::Parser;
use arc_cli::{run, Args};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    run(&args);
    Ok(())
}
