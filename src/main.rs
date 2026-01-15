use clap::Parser;
use console::style;
use arc_cli::{CliArgs, run};

#[tokio::main]
async fn main() {
    // Ensure rustls (used by the kube crate for TLS) uses the default crypto provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");

    let args = CliArgs::try_parse().unwrap_or_else(|e| {
        eprintln!("{}", style(e).red());
        std::process::exit(1);
    });

    if let Err(e) = run(args).await {
        // Suppress errors from someone hitting Esc during
        // prompts because cliclack already displays a message
        if e.to_string() != "IO error: operation interrupted" {
            eprintln!("{}", style(e).red());
            std::process::exit(1);
        }
    };
}
