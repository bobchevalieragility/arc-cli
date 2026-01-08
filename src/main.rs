use clap::Parser;
use arc_cli::{run, Args};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure rustls (used by the kube crate for TLS) uses the default crypto provider
    let _ = rustls::crypto::ring::default_provider().install_default();

    let args = Args::try_parse().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    run(&args).await;
    Ok(())
}
