# Git Commit Creator (gitcc) 🚀

A beautiful command-line tool built with Rust and `ratatui` for streamlined git operations. Create conventional commits and push changes with an interactive TUI interface.

## Features

- 🎨 **Beautiful TUI Interface** - Interactive terminal UI built with `ratatui`
- 📝 **Conventional Commits** - Support for standard commit prefixes (feat, fix, docs, etc.)
- 🎯 **Selective Staging** - Stage files by extension or directory
- 🔍 **File Preview** - Review staged files before committing
- 🎫 **ROKU Ticket Support** - Automatic ticket extraction from branch names
- ⚡ **Fast & Efficient** - Built in Rust for performance

## Installation

### Prerequisites

- Rust 1.70+ 
- Git repository

### Build from Source

```bash
git clone <repository-url>
cd gitcc
cargo build --release
```

The binary will be available at `target/release/gitcc`.

## Usage

### Interactive Mode (Default)

```bash
gitcc
```

This launches the TUI interface where you can:
1. Review staged files
2. Select commit prefix interactively
3. Enter commit message
4. Confirm and push

### Command Line Arguments

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

### Available Options

- `-m, --message <MESSAGE>` - Commit message (omit for interactive prompt)
- `-p, --prefix <PREFIX>` - Commit prefix (omit for interactive prompt)
- `--no-push` - Do not push after committing
- `-e, --extensions <EXTENSIONS>` - Comma-separated file extensions to stage
- `-d, --directory <DIRECTORY>` - Directory to restrict staging to
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
- `ROKU-` - Special prefix for ROKU tickets (extracts ticket number from branch)

## ROKU Ticket Integration

When selecting `ROKU-` prefix, the tool automatically:
1. Extracts ticket number from current branch name (e.g., `feature/ROKU-12345-new-feature`)
2. Uses the ticket number as prefix (e.g., `ROKU-12345: your message`)

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

## Examples

### Basic Usage
```bash
# Interactive mode - launches TUI
gitcc

# Quick commit with CLI args
gitcc -p "fix:" -m "resolve login issue"
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

### ROKU Workflow
```bash
# On branch: feature/ROKU-12345-user-auth
gitcc -p "ROKU-" -m "implement user authentication"
# Results in commit: "ROKU-12345: implement user authentication"
```

## Error Handling

The tool provides clear error messages for common issues:
- Not in a git repository
- No files staged
- Failed git operations
- Invalid ROKU ticket extraction

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
- `regex` - Pattern matching for ROKU tickets

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

[Add your license here]

## Contributing

[Add contribution guidelines here]