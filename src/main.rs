//! Git Commit Creator (gitcc) - A TUI tool for creating conventional commits

mod git;
mod key_handler;
mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, poll},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::Duration;

/// Command line interface configuration
#[derive(Parser)]
#[command(name = "gitcc")]
#[command(about = "Git Commit Creator – stage, commit and push with ease")]
struct Cli {
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
    "ROKU-",
];

/// Application state for TUI navigation
#[derive(Debug, Clone)]
pub enum AppState {
    StagedFilesReview,
    PrefixSelection,
    MessageInput,
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
}

impl App {
    /// Creates new App instance with initial state based on CLI arguments
    fn new(prefix: Option<String>, message: Option<String>, no_push: bool) -> Self {
        let state = AppState::StagedFilesReview;

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

    let mut app = App::new(cli.prefix, cli.message, cli.no_push);
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
