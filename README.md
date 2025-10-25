# Git Commit Creator (gitcc) 🚀

A beautiful command-line tool built with Rust and `ratatui` for streamlined git operations. Create conventional commits, create branches, and push changes with an interactive TUI interface.

## Features

- 🎨 **Beautiful TUI Interface** - Interactive terminal UI built with `ratatui`
- 📝 **Conventional Commits** - Support for standard commit prefixes (feat, fix, docs, etc.)
- 🌿 **Branch Creation** - Create branches with conventional prefixes and story numbers
- 🎯 **Interactive File Staging** - Navigate and selectively stage/unstage files
- 🔍 **File Status Indicators** - See at a glance if files are Added (A), Modified (M), or Deleted (D)
- 📋 **File Preview** - Review staged files before committing
- ⚙️ **Configurable** - Customize commit prefixes, branch prefixes, and more via config file
- ⚡ **Fast & Efficient** - Built in Rust for performance

## Installation

### Prerequisites

- Rust 1.70+ 
- Git repository

### Install from crates.io

```bash
cargo install gitcc
```

### Build from Source

```bash
git clone https://github.com/harrisonoest/gitcc
cd gitcc
cargo build --release
```

The binary will be available at `target/release/gitcc`.

## Usage

### Interactive Mode (Default)

#### Commit Mode
```bash
gitcc
```

This launches the TUI interface where you can:
1. Review staged files
2. Select commit prefix interactively
3. Enter commit message
4. Confirm and push

#### Branch Creation Mode
```bash
gitcc --branch
```

This launches the TUI interface for branch creation where you can:
1. Select branch prefix interactively
2. Enter optional Jira story number
3. Enter branch name
4. Create and checkout branch

### Command Line Arguments

#### Commit Operations
```bash
# Specify commit message and prefix
gitcc -p "feat:" -m "add new feature"

# Stage specific file extensions
gitcc -e "rs,toml" -m "update rust files"

# Stage files from specific directory
gitcc -d "src/" -p "refactor:" -m "restructure source code"

# Skip pushing after commit
gitcc --no-push -m "local commit only"
```

#### Branch Operations
```bash
# Interactive branch creation
gitcc --branch

# Create branch with all parameters
gitcc --branch --branch-prefix "feat" --story "123" --branch-name "new-feature"

# Create branch without story number
gitcc --branch --branch-prefix "fix" --branch-name "bug-fix"
```

### Configuration

Gitcc uses a configuration file located at `~/.gitcc/config.toml`. The file is automatically created with default values on first run.

### Configuration File Structure

The configuration file uses TOML format and supports the following options:

```toml
# Custom commit prefixes - array of strings
commit_prefixes = ["feat:", "fix:", "docs:", "style:", "refactor:", "test:", "ci:", "chore:"]

# Custom branch prefixes - array of strings
branch_prefixes = ["build", "chore", "ci", "docs", "feat", "fix", "perf", "refactor", "revert", "style", "test"]

# Optional story number prefix (e.g., "JIRA-", "TICKET-") - optional string
story_prefix = "JIRA-"

# Auto-push commits after creation - optional boolean (default: true)
auto_push = true

# Default commit prefix to pre-select - optional string
default_commit_prefix = "feat:"
```

### Configuration Options Explained

- **`commit_prefixes`**: Array of commit prefixes shown in the TUI selection. Add or remove prefixes to match your team's conventions.
- **`branch_prefixes`**: Array of branch prefixes for branch creation mode. Customize based on your branching strategy.
- **`story_prefix`**: Optional prefix for story numbers (e.g., "JIRA-123" becomes "JIRA-123: commit message"). Set to `null` or omit to disable.
- **`auto_push`**: Whether to automatically push commits after creation. Set to `false` to commit locally only by default.
- **`default_commit_prefix`**: Pre-select a specific commit prefix in the TUI. Must match one of the prefixes in `commit_prefixes`.

### Modifying Configuration

1. **Location**: The config file is located at `~/.gitcc/config.toml`
2. **Auto-creation**: If the file doesn't exist, gitcc creates it with default values on first run
3. **Manual editing**: Edit the file with any text editor
4. **Validation**: Invalid TOML syntax will cause gitcc to fail with an error message
5. **Reloading**: Changes take effect the next time you run gitcc

### Example Customizations

**For a team using Jira tickets:**
```toml
commit_prefixes = ["feat:", "fix:", "docs:", "refactor:", "test:"]
story_prefix = "JIRA-"
default_commit_prefix = "feat:"
```

**For a team that doesn't push automatically:**
```toml
auto_push = false
commit_prefixes = ["add:", "fix:", "update:", "remove:"]
```

**For custom branch workflow:**
```toml
branch_prefixes = ["feature", "bugfix", "hotfix", "release"]
story_prefix = "TICKET-"
```

## Available Options

#### Commit Options
- `-m, --message <MESSAGE>` - Commit message (omit for interactive prompt)
- `-p, --prefix <PREFIX>` - Commit prefix (omit for interactive prompt)
- `--no-push` - Do not push after committing
- `-e, --extensions <EXTENSIONS>` - Comma-separated file extensions to stage
- `-d, --directory <DIRECTORY>` - Directory to restrict staging to

#### Branch Options
- `-b, --branch` - Enable branch creation mode
- `--branch-prefix <PREFIX>` - Branch prefix (omit for interactive prompt)
- `--story <NUMBER>` - Story number (optional)
- `--branch-name <NAME>` - Branch name (omit for interactive prompt)

#### General Options
- `-h, --help` - Show help information

## Commit Prefixes

The tool supports conventional commit prefixes:

- `feat:` - New features
- `fix:` - Bug fixes  
- `docs:` - Documentation changes
- `style:` - Code style changes
- `refactor:` - Code refactoring
- `test:` - Test additions/changes
- `chore:` - Maintenance tasks

## Branch Prefixes

The tool supports conventional branch prefixes:

- `build` - Build system changes
- `chore` - Maintenance tasks
- `ci` - CI/CD changes
- `docs` - Documentation changes
- `feat` - New features
- `fix` - Bug fixes
- `perf` - Performance improvements
- `refactor` - Code refactoring
- `revert` - Revert changes
- `style` - Code style changes
- `test` - Test additions/changes

Branch names are created in the format: `prefix/{story_prefix}{story}/{name}` or `prefix/{name}` if no story number is provided. The `story_prefix` comes from your configuration file.

## TUI Controls

### File Review Screen
- `↑/↓` - Navigate files
- `Space` - Stage/unstage selected file
- `j/k` - Scroll diff preview
- `Enter` - Proceed with staged files
- `Esc` - Quit

**File Status Indicators:**
- `[A]` - Added (new file)
- `[M]` - Modified (existing file changed)
- `[D]` - Deleted (file removed)

Files are displayed as: `[S] [A] filename` where `[S]` indicates staged status and `[A]` indicates file status.

### Prefix Selection Screen  
- `↑/↓` - Navigate options
- `Enter` - Select prefix
- `Esc` - Quit

### Message Input Screen
- Type your commit message
- `Enter` - Confirm message
- `Ctrl/Alt + Backspace` - Delete word backward
- `Alt + D` - Delete word forward
- `Ctrl/Alt + ←/→` - Move by word
- `Home/End` - Move to start/end
- `Esc` - Quit

### Branch Creation Screens
- `↑/↓` - Navigate branch prefixes
- `Enter` - Select/confirm input
- Type story number (digits only) or branch name
- `Esc` - Quit at any time

## Examples

### Basic Usage
```bash
# Interactive commit mode - launches TUI
gitcc

# Interactive branch creation mode - launches TUI
gitcc --branch

# Quick commit with CLI args
gitcc -p "fix:" -m "resolve login issue"

# Quick branch creation with CLI args
gitcc --branch --branch-prefix "feat" --story "456" --branch-name "user-authentication"
```

### Selective Staging
```bash
# Stage only Rust files
gitcc -e "rs" -p "refactor:" -m "improve error handling"

# Stage files in src directory
gitcc -d "src/" -p "feat:" -m "add new module"

# Combine directory and extensions
gitcc -d "tests/" -e "rs" -p "test:" -m "add integration tests"
```

## Error Handling

The tool provides clear error messages for common issues:
- Not in a git repository
- No files staged
- Failed git operations

If an error occurs, all staged changes are automatically unstaged to maintain a clean git state.

## Project Structure

```
src/
├── main.rs        # Main application logic and CLI setup
├── git.rs         # Git operations (staging, committing, pushing)
├── ui.rs          # TUI rendering and interface components
└── key_handler.rs # Keyboard input handling
```

## Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal manipulation
- `clap` - Command line argument parsing
- `git2` - Git operations
- `tokio` - Async runtime
- `anyhow` - Error handling
- `serde` - Configuration serialization
- `toml` - Configuration file format
- `dirs` - Home directory detection

## Development

```bash
# Build the project
cargo build

# Run with clippy for linting
cargo clippy -- -D warnings

# Run tests
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

[Add contribution guidelines here]
