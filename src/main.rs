use clap::Parser;
use cliclack::outro;
use std::process::{Command, Stdio};
use std::env;
use arc_cli::{run, Args};


// #[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args_os: Vec<std::ffi::OsString> = env::args_os().collect();
    let args = Args::parse();
    run(&args, &args_os);

    // let selected_profile = aws_profile(args.profile).await?;
    //
    // // 1. Determine the shell to use.
    // // Kubie detects the current shell, but you can default to 'sh' or 'bash'.
    // // A simple way is to check the SHELL environment variable.
    // let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
    // println!("Detected shell: {}", shell);
    //
    // // 2. Spawn the shell process.
    // let mut command = Command::new(&shell);
    //
    // // 3. Set the desired environment variable(s) for the new shell session.
    // // This mimics how kubie sets KUBECONFIG or other specific variables.
    // // For example, setting a dummy KUBECONFIG to a specific file:
    // command.env("AWS_PROFILE", selected_profile);
    // command.env("AREPL_ACTIVE", "true");
    //
    // // 4. Configure I/O to make it an interactive session.
    // // This connects the new shell's input/output to the current terminal.
    // command.stdin(Stdio::inherit())
    //     .stdout(Stdio::inherit())
    //     .stderr(Stdio::inherit());
    //
    // // 5. Spawn the process.
    // // The `spawn` call creates the new process. We then wait for it to complete.
    // println!("Spawning new shell with custom environment...");
    //
    // match command.spawn() {
    //     Ok(mut child) => {
    //         // Wait for the child process to finish.
    //         let status = child.wait().expect("Failed to wait on child process");
    //         println!("Shell exited with status: {}", status);
    //     }
    //     Err(e) => {
    //         eprintln!("Failed to spawn shell '{}': {}", shell, e);
    //     }
    // }

    Ok(())
}
