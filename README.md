# Git Commit Creator (gitcc) 🚀

A beautiful command-line tool built with Rust and `ratatui` for streamlined git operations. Create conventional commits, create branches, and push changes with an interactive TUI interface.

## Features

- 🎨 **Beautiful TUI Interface** - Interactive terminal UI built with `ratatui`
- 📝 **Conventional Commits** - Support for standard commit prefixes (feat, fix, docs, etc.)
- 🌿 **Branch Creation** - Create branches with conventional prefixes and Jira story numbers
- 🎯 **Selective Staging** - Stage files by extension or directory
- 🔍 **File Preview** - Review staged files before committing
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

### Available Options

#### Commit Options
- `-m, --message <MESSAGE>` - Commit message (omit for interactive prompt)
- `-p, --prefix <PREFIX>` - Commit prefix (omit for interactive prompt)
- `--no-push` - Do not push after committing
- `-e, --extensions <EXTENSIONS>` - Comma-separated file extensions to stage
- `-d, --directory <DIRECTORY>` - Directory to restrict staging to

#### Branch Options
- `-b, --branch` - Enable branch creation mode
- `--branch-prefix <PREFIX>` - Branch prefix (omit for interactive prompt)
- `--story <NUMBER>` - Jira story number (optional)
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

Branch names are created in the format: `prefix/wuko-{story}/{name}` or `prefix/{name}` if no story number is provided.

## TUI Controls

### Staged Files Screen
- `y` - Proceed with commit
- `n` - Abort and unstage changes
- `Esc` - Quit

### Prefix Selection Screen  
- `↑/↓` - Navigate options
- `Enter` - Select prefix
- `Esc` - Quit

### Message Input Screen
- Type your commit message
- `Enter` - Confirm message
- `Backspace` - Delete characters
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
