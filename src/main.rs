//! Git Commit Creator (gitcc) - A TUI tool for creating conventional commits

mod config;
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

/// Cursor blink interval in milliseconds
const CURSOR_BLINK_MS: u64 = 500;

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

    #[arg(short, long, help = "Search for branches by substring")]
    search: Option<String>,
}

/// Application state for TUI navigation
#[derive(Debug, Clone)]
pub enum AppState {
    StagedFilesReview,
    PrefixSelection,
    MessageInput,
    BranchPrefixSelection,
    BranchStoryInput,
    BranchNameInput,
    BranchSearch,
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
    pub commit_prefixes: Vec<String>,
    pub branch_prefixes: Vec<String>,
    pub is_branch_mode: bool,
    pub selected_branch_prefix_index: usize,
    pub branch_story: String,
    pub branch_name: String,
    pub branch_prefix: Option<String>,
    pub filter: String,
    pub file_statuses: std::collections::HashMap<String, git::FileStatus>,
    pub current_diff: String,
    pub diff_scroll_offset: usize,
    pub diff_visible_lines: usize,
    pub search_query: String,
    pub matching_branches: Vec<String>,
    pub selected_branch_index: usize,
}

impl App {
    /// Creates new App instance with initial state based on CLI arguments
    fn new(
        prefix: Option<String>,
        message: Option<String>,
        no_push: bool,
        is_branch_mode: bool,
        branch_prefix: Option<String>,
        app_config: &config::Config,
    ) -> Self {
        let state = if is_branch_mode {
            AppState::BranchPrefixSelection
        } else {
            AppState::StagedFilesReview
        };

        let cursor_pos = message.as_ref().map_or(0, |m| m.len());

        // Find default prefix index if configured
        let default_prefix_index = app_config
            .default_commit_prefix
            .as_ref()
            .and_then(|default| app_config.commit_prefixes.iter().position(|p| p == default))
            .unwrap_or(0);

        let mut app = Self {
            state,
            staged_files: Vec::new(),
            selected_prefix_index: default_prefix_index,
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
            commit_prefixes: app_config.commit_prefixes.clone(),
            branch_prefixes: app_config.branch_prefixes.clone(),
            is_branch_mode,
            selected_branch_prefix_index: 0,
            branch_story: String::new(),
            branch_name: String::new(),
            branch_prefix,
            filter: String::new(),
            file_statuses: std::collections::HashMap::new(),
            current_diff: String::new(),
            diff_scroll_offset: 0,
            diff_visible_lines: 0,
            search_query: String::new(),
            matching_branches: Vec::new(),
            selected_branch_index: 0,
        };

        // Reset filter and selection for branch mode
        if is_branch_mode {
            app.filter.clear();
            app.selected_branch_prefix_index = 0;
        }

        app
    }

    /// Get filtered commit prefixes
    pub fn filtered_commit_prefixes(&self) -> Vec<String> {
        if self.filter.is_empty() {
            self.commit_prefixes.clone()
        } else {
            self.commit_prefixes
                .iter()
                .filter(|prefix| prefix.to_lowercase().contains(&self.filter.to_lowercase()))
                .cloned()
                .collect()
        }
    }

    /// Get filtered branch prefixes
    pub fn filtered_branch_prefixes(&self) -> Vec<String> {
        if self.filter.is_empty() {
            self.branch_prefixes.clone()
        } else {
            self.branch_prefixes
                .iter()
                .filter(|prefix| prefix.to_lowercase().contains(&self.filter.to_lowercase()))
                .cloned()
                .collect()
        }
    }

    /// Update the current diff for the selected file
    pub fn update_current_diff(&mut self) {
        if let Some(file) = self.all_files.get(self.selected_file_index) {
            let is_staged = self.staged_files_set.contains(file);
            self.current_diff = git::get_file_diff(file, is_staged)
                .unwrap_or_else(|_| "Error fetching diff".to_string());
            self.diff_scroll_offset = 0;
        } else {
            self.current_diff = String::new();
            self.diff_scroll_offset = 0;
        }
    }
}

/// Main TUI event loop
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<App> {
    loop {
        terminal.draw(|f| ui::render(f, &mut app))?;

        if app.should_quit {
            break;
        }

        if poll(Duration::from_millis(CURSOR_BLINK_MS))? {
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

/// Sets up terminal for TUI mode, runs the provided function, then restores terminal
fn with_terminal<F, T>(f: F) -> Result<T>
where
    F: FnOnce(&mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<T>,
{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = f(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn main() -> Result<()> {
    let app_config = config::Config::load()?;
    let cli = Cli::parse();

    // Handle branch search mode
    if let Some(query) = cli.search {
        return handle_branch_search(query);
    }

    // Handle branch creation mode
    if cli.branch {
        return handle_branch_creation(cli, &app_config);
    }

    let repo = git::ensure_git_repository()?;

    // Check if there are any changes to stage
    if !git::has_changes(&repo)? {
        println!("✨ No changes to commit. Working directory is clean.");
        return Ok(());
    }

    git::stage_files(cli.extensions, cli.directory)?;
    let (all_files, staged_files, file_statuses) = git::get_all_changed_files(&repo)?;

    if staged_files.is_empty() {
        println!("❌ No files staged. Aborting.");
        return Ok(());
    }

    // Determine no_push from CLI flag or config (CLI takes precedence)
    let no_push = cli.no_push || !app_config.auto_push.unwrap_or(true);

    let mut app = App::new(cli.prefix, cli.message, no_push, false, None, &app_config);
    app.all_files = all_files;
    app.staged_files = staged_files.clone();
    app.staged_files_set = staged_files.into_iter().collect();
    app.file_statuses = file_statuses;
    app.update_current_diff();

    let result = with_terminal(|terminal| run_app(terminal, app));

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

fn handle_branch_creation(cli: Cli, app_config: &config::Config) -> Result<()> {
    git::ensure_git_repository()?;

    // If all branch parameters provided via CLI, create directly
    if let (Some(prefix), Some(name)) = (&cli.branch_prefix, &cli.branch_name) {
        let branch_name = git::build_branch_name(
            prefix,
            cli.story.as_deref(),
            name,
            app_config.story_prefix.as_deref(),
        )?;
        git::create_and_checkout_branch(&branch_name)?;
        return Ok(());
    }

    // Otherwise, use interactive mode
    let mut app = App::new(None, None, false, true, cli.branch_prefix, app_config);
    if let Some(story) = cli.story {
        app.branch_story = story;
    }
    if let Some(name) = cli.branch_name {
        app.branch_name = name;
    }

    let result = with_terminal(|terminal| run_app(terminal, app));

    match result {
        Ok(app) => {
            if app.should_proceed && app.branch_prefix.is_some() && !app.branch_name.is_empty() {
                let story = if app.branch_story.is_empty() {
                    None
                } else {
                    Some(app.branch_story.as_str())
                };
                let branch_name = git::build_branch_name(
                    &app.branch_prefix.unwrap(),
                    story,
                    &app.branch_name,
                    app_config.story_prefix.as_deref(),
                )?;
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

fn handle_branch_search(query: String) -> Result<()> {
    git::ensure_git_repository()?;

    let all_branches = git::get_all_branches()?;
    let matching = git::search_branches(&query, &all_branches);

    let mut app = App::new(None, None, false, false, None, &config::Config::load()?);
    app.state = AppState::BranchSearch;
    app.search_query = query;
    app.matching_branches = matching;

    let result = with_terminal(|terminal| run_app(terminal, app));

    match result {
        Ok(app) => {
            if app.should_proceed && !app.matching_branches.is_empty() {
                let selected = &app.matching_branches[app.selected_branch_index];
                git::checkout_branch(selected)?;
            } else {
                println!("⏹️ Branch search aborted by user.");
            }
        }
        Err(e) => {
            println!("❌ Error: {e}");
        }
    }

    Ok(())
}
