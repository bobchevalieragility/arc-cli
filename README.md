# arc-cli
This CLI tool unifies functionality from multiple tools (kubectl, awscli, pgcli, vault, etc.) into a single interface tailored for Arc Backend developers. It also replaces functionality that would typically be provided by shell functions/scripts.

![Help Menu](assets/help_menu.png)

## Features
- Command dependencies are automatically handled. For instance:
  - If a command needs port-forwarding, a session will be automatically started/stopped
  - If a command requires a secret, the secret will be automatically fetched from either Vault or AWS Secrets Manager
  - If a command interacts with AWS and you don't have an active AWS profile, you'll be prompted to select one
- If command args aren't explicitly provided, the user is prompted to interactively select from a menu.\
  (So you don't have to remember every profile name, k8s context, service port, etc.)
- Selection menus are context-aware, meaning values are filtered based on previously specified inputs.
- Terminal isolation is enforced for Kubernetes contexts, meaning that multiple terminal sessions, with different contexts, can be open simultaneously.
- It's built with Rust, ensuring high performance and reliability.

## Examples
### Dynamically modify logging level of an Arc Backend service
In addition to the desired logging level, the `log-level` command also needs to be told which service to modify, in which K8 cluster the service resides, and a port-forwarding session to the service must exist.  If any of this context does not exist in the current "state" of the program, the corresponding commands to gather that context will be automatically executed before the `log-level` command. Once the overall program execution completes, the port-forwarding session is automatically torn down.

![pgcli](assets/demo-log-level.gif)

### Launch pgcli  
Launching `pgcli` depends on knowing which AWS account to use and which instance of Postres to connect to. If any of that info is not explicitly provided, the corresponding commands to gather that context will be automatically executed, resulting in the user being prompted to provide the necessary context.

![pgcli](assets/demo-pgcli.gif)

### Switch active AWS Profile and/or K8s Context
The available AWS Profiles are inferred by inspecting ~/.aws/config.  Similarly, the available K8 Contexts are inferred by inspecting ~/.kube/config.

![pgcli](assets/demo-switch.gif)

## Prerequisites
- [pgcli](https://www.pgcli.com/) (if utilizing the `pgcli` command) 
   ```bash
   brew install pgcli
   ```

## Installation

### Option 1: From Source
1. Install Rust (see https://www.rust-lang.org/tools/install for alternate methods)
   ```bash
   curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
   ```
2. Download the source code:
   ```bash
   git clone git@github.com:bobchevalieragility/arc-cli.git
   ```
3. From the root of the project, build and install the binary:   
   ```bash
   cargo install --path .
   ```
4. Install the wrapper script:  
   ```bash
   mkdir ~/.arc-cli
   cp scripts/arc.sh ~/.arc-cli/arc.sh
   ```
5. Source the wrapper script from your shell config file (.zshrc, .bashrc, etc.):
   ```bash
   echo 'source ~/.arc-cli/arc.sh' >> ~/.zshrc
   ```
### Option 2: Pre-compiled Binaries
1. Choose the binary type that is appropriate for your OS:
   - MacOS ARM64: `arc-macos-arm64`
   - MacOS x86_64: `arc-macos-amd64`
   - Linux x86_64: `arc-linux-amd64`
2. Download and install the latest version of the binary: 
   ```bash
   curl -LO https://github.com/bobchevalieragility/arc-cli/releases/latest/download/arc-macos-arm64
   chmod +x arc-macos-arm64
   sudo mv arc-macos-arm64 /usr/local/bin/arc
   ```
3. Download and install the latest version of the wrapper script:
   ```bash
   curl -LO https://github.com/bobchevalieragility/arc-cli/releases/latest/download/arc.sh
   chmod +x arc.sh
   mkdir ~/.arc-cli
   mv arc.sh ~/.arc-cli/arc.sh
   ```
4. Source the wrapper script from your shell config file (.zshrc, .bashrc, etc.):
   ```bash
   echo 'source ~/.arc-cli/arc.sh' >> ~/.zshrc
   ```

## Development

### Commit Message Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/) to automatically generate changelogs and determine version bumps. When you commit changes, use the following format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

#### Commit Types and Changelog Groups

The following table shows which commit prefixes appear in the changelog and how they affect versioning:

| Commit Prefix | Changelog Group        | Semantic Version Impact   | Example Commit Message                    |
|---------------|------------------------|---------------------------|-------------------------------------------|
| `feat`        | ⛰️ Features            | **Minor** (0.1.0 → 0.2.0) | `feat: add vault secret retrieval`        |
| `fix`         | 🐛 Bug Fixes           | **Patch** (0.1.0 → 0.1.1) | `fix: resolve port forwarding timeout`    |
| `perf`        | ⚡ Performance          | **Patch** (0.1.0 → 0.1.1) | `perf: optimize kube API calls`           |
| `refactor`    | 🚜 Refactor            | No version bump           | `refactor: simplify task execution logic` |
| `doc`         | 📚 Documentation       | No version bump           | `doc: update installation instructions`   |
| `style`       | 🎨 Styling             | No version bump           | `style: format code with rustfmt`         |
| `test`        | 🧪 Testing             | No version bump           | `test: add integration tests for RDS`     |
| `chore`       | ⚙️ Miscellaneous Tasks | No version bump           | `chore: update dependencies`              |
| `ci`          | ⚙️ Miscellaneous Tasks | No version bump           | `ci: fix release workflow`                |
| `revert`      | ◀️ Revert              | No version bump           | `revert: undo previous commit`            |

#### Breaking Changes

To trigger a **Major** version bump (0.1.0 → 1.0.0), add `BREAKING CHANGE:` in the commit body or footer:

```
feat: redesign CLI arguments

BREAKING CHANGE: All command arguments have been restructured
```

Or use an exclamation mark after the type/scope:

```
feat!: redesign CLI arguments
```

#### Commits Excluded from Changelog

The following commit types are automatically excluded from the changelog:

- `chore(release):` - Release commits
- `chore(deps)` - Dependency updates  
- `chore(pr)` / `chore(pull)` - PR maintenance
- `refactor(clippy)` - Clippy suggestions

#### Scopes (Optional)

You can add a scope to provide additional context:

```
feat(vault): add secret caching
fix(kube): handle connection timeout
docs(readme): add contribution guidelines
```

### Release Process

This project uses [release-plz](https://release-plz.iem.at/) to automate releases:

1. **Merge a PR to `main`** - Use conventional commit messages
2. **Automated PR is created** - release-plz analyzes commits and creates a PR with:
   - Updated version in `Cargo.toml`
   - Generated changelog in `CHANGELOG.md`
3. **Review and merge the PR** - Once merged:
   - A Git tag is created with the new version
   - A GitHub Release is created and associated with the new tag
   - Binaries are built for multiple architectures and uploaded to Release

No manual version bumping or changelog editing required!

