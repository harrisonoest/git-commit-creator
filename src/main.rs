//! Git Commit Creator (gitcc) - A TUI tool for creating conventional commits

mod git;
mod key_handler;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, poll, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

/// Command line interface configuration
#[derive(Parser)]
#[command(name = "gitcc")]
#[command(about = "Git Commit Creator – stage, commit and push with ease")]
#[command(version)]
#[command(disable_version_flag = true)]
struct Cli {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version, help = "Print version")]
    _version: (),
    #[arg(short, long, help = "Commit message (omit for interactive prompt)")]
    message: Option<String>,

    #[arg(short, long, help = "Commit prefix (omit for interactive prompt)")]
    prefix: Option<String>,

    #[arg(long, help = "Do not push after committing")]
    no_push: bool,

    #[arg(short, long, help = "Comma-separated list of file extensions to stage")]
    extensions: Option<String>,

    #[arg(short, long, help = "Directory to restrict staging to")]
    directory: Option<String>,

    #[arg(short, long, help = "Create a new branch interactively")]
    branch: bool,

    #[arg(long, help = "Branch prefix for new branch")]
    branch_prefix: Option<String>,

    #[arg(long, help = "Jira story number for branch")]
    story: Option<String>,

    #[arg(long, help = "Branch name")]
    branch_name: Option<String>,
}

/// Available conventional commit prefixes
pub const COMMIT_PREFIXES: &[&str] = &[
    "feat:",
    "fix:",
    "docs:",
    "style:",
    "refactor:",
    "test:",
    "ci:",
    "chore:",
];

/// Available branch prefixes
pub const BRANCH_PREFIXES: &[&str] = &[
    "build", "chore", "ci", "docs", "feat", "fix", "perf", "refactor", "revert", "style", "test",
];

/// Application state for TUI navigation
#[derive(Debug, Clone)]
pub enum AppState {
    StagedFilesReview,
    PrefixSelection,
    MessageInput,
    BranchPrefixSelection,
    BranchStoryInput,
    BranchNameInput,
}

/// Main application state
pub struct App {
    pub state: AppState,
    pub staged_files: Vec<String>,
    pub selected_prefix_index: usize,
    pub commit_message: String,
    pub should_quit: bool,
    pub should_proceed: bool,
    pub prefix: Option<String>,
    pub message: Option<String>,
    pub no_push: bool,
    pub cursor_visible: bool,
    pub cursor_position: usize,
    pub selected_file_index: usize,
    pub all_files: Vec<String>,
    pub staged_files_set: std::collections::HashSet<String>,
    pub is_branch_mode: bool,
    pub selected_branch_prefix_index: usize,
    pub branch_story: String,
    pub branch_name: String,
    pub branch_prefix: Option<String>,
}

impl App {
    /// Creates new App instance with initial state based on CLI arguments
    fn new(
        prefix: Option<String>,
        message: Option<String>,
        no_push: bool,
        is_branch_mode: bool,
        branch_prefix: Option<String>,
    ) -> Self {
        let state = if is_branch_mode {
            AppState::BranchPrefixSelection
        } else {
            AppState::StagedFilesReview
        };

        let cursor_pos = message.as_ref().map_or(0, |m| m.len());

        Self {
            state,
            staged_files: Vec::new(),
            selected_prefix_index: 0,
            commit_message: message.clone().unwrap_or_default(),
            should_quit: false,
            should_proceed: false,
            prefix,
            message,
            no_push,
            cursor_visible: true,
            cursor_position: cursor_pos,
            selected_file_index: 0,
            all_files: Vec::new(),
            staged_files_set: std::collections::HashSet::new(),
            is_branch_mode,
            selected_branch_prefix_index: 0,
            branch_story: String::new(),
            branch_name: String::new(),
            branch_prefix,
        }
    }
}

/// Main TUI event loop
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<App> {
    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if app.should_quit {
            break;
        }

        if poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                key_handler::handle_key(&mut app, key.code, key.modifiers);
            }
        } else {
            // Toggle cursor visibility for blinking effect
            app.cursor_visible = !app.cursor_visible;
        }
    }

    Ok(app)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle branch creation mode
    if cli.branch {
        return handle_branch_creation(cli).await;
    }

    let repo = git::ensure_git_repository()?;

    // Check if there are any changes to stage
    if !git::has_changes(&repo)? {
        println!("✨ No changes to commit. Working directory is clean.");
        return Ok(());
    }

    git::stage_files(cli.extensions, cli.directory)?;
    let (all_files, staged_files) = git::get_all_changed_files(&repo)?;

    if staged_files.is_empty() {
        println!("❌ No files staged. Aborting.");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(cli.prefix, cli.message, cli.no_push, false, None);
    app.all_files = all_files;
    app.staged_files = staged_files.clone();
    app.staged_files_set = staged_files.into_iter().collect();

    let result = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match result {
        Ok(app) => {
            if app.should_proceed && app.prefix.is_some() && app.message.is_some() {
                let commit_msg =
                    git::build_commit_message(&app.prefix.unwrap(), &app.message.unwrap())?;
                git::commit_and_push(&commit_msg, app.no_push)?;
            } else {
                println!("⏹️ Aborted by user. Unstaging changes...");
                git::unstage_all()?;
            }
        }
        Err(e) => {
            println!("❌ Error: {e}");
            git::unstage_all()?;
        }
    }

    Ok(())
}

async fn handle_branch_creation(cli: Cli) -> Result<()> {
    git::ensure_git_repository()?;

    // If all branch parameters provided via CLI, create directly
    if let (Some(prefix), Some(name)) = (&cli.branch_prefix, &cli.branch_name) {
        let branch_name = git::build_branch_name(prefix, cli.story.as_deref(), name)?;
        git::create_and_checkout_branch(&branch_name)?;
        return Ok(());
    }

    // Otherwise, use interactive mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(None, None, false, true, cli.branch_prefix);
    if let Some(story) = cli.story {
        app.branch_story = story;
    }
    if let Some(name) = cli.branch_name {
        app.branch_name = name;
    }

    let result = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match result {
        Ok(app) => {
            if app.should_proceed && app.branch_prefix.is_some() && !app.branch_name.is_empty() {
                let story = if app.branch_story.is_empty() {
                    None
                } else {
                    Some(app.branch_story.as_str())
                };
                let branch_name =
                    git::build_branch_name(&app.branch_prefix.unwrap(), story, &app.branch_name)?;
                git::create_and_checkout_branch(&branch_name)?;
            } else {
                println!("⏹️ Branch creation aborted by user.");
            }
        }
        Err(e) => {
            println!("❌ Error: {e}");
        }
    }

    Ok(())
}
