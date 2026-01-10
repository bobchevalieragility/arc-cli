use anyhow::Result;
use clap::Parser;
use arc_cli::{run, Args};

#[tokio::main]
async fn main() -> Result<()> {
    // Ensure rustls (used by the kube crate for TLS) uses the default crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("Failed to install default crypto provider"))?;

    let args = Args::try_parse().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    run(&args).await.map_err(Into::into)
}
