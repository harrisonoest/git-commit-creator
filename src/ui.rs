use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{App, AppState, BRANCH_PREFIXES, COMMIT_PREFIXES};

/// Renders the TUI interface based on current application state
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area());

    let title = if app.is_branch_mode {
        Paragraph::new("Git Branch Creator (gitcc) 🌿")
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL))
    } else {
        Paragraph::new("Git Commit Creator (gitcc) 🚀")
            .style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL))
    };
    f.render_widget(title, chunks[0]);

    match app.state {
        AppState::StagedFilesReview => {
            let items: Vec<ListItem> = app
                .all_files
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let is_staged = app.staged_files_set.contains(f);
                    let prefix = if is_staged { "[S]" } else { "[ ]" };
                    let style = if i == app.selected_file_index {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else if is_staged {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };
                    ListItem::new(format!("{prefix} {f}")).style(style)
                })
                .collect();

            let files_list = List::new(items)
                .block(
                    Block::default()
                        .title("📁 Files (Enter to stage/unstage)")
                        .borders(Borders::ALL),
                )
                .style(Style::default());

            let help = if app.staged_files_set.is_empty() {
                Paragraph::new("⚠️ No files staged - stage at least one file to proceed")
                    .style(Style::default().fg(Color::Red))
                    .wrap(Wrap { trim: true })
            } else {
                Paragraph::new(
                    "↑↓ to navigate, Enter to stage/unstage, 'y' to proceed, Esc to abort",
                )
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true })
            };

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                .split(chunks[1]);

            f.render_widget(files_list, layout[0]);
            f.render_widget(help, layout[1]);
        }
        AppState::PrefixSelection => {
            let items: Vec<ListItem> = COMMIT_PREFIXES
                .iter()
                .enumerate()
                .map(|(i, prefix)| {
                    let style = if i == app.selected_prefix_index {
                        Style::default().bg(Color::DarkGray).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    ListItem::new(*prefix).style(style)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title("Select Commit Prefix")
                    .borders(Borders::ALL),
            );

            let help = Paragraph::new("Use ↑↓ to navigate, Enter to select, Esc to quit")
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                .split(chunks[1]);

            f.render_widget(list, layout[0]);
            f.render_widget(help, layout[1]);
        }
        AppState::MessageInput => {
            let message_with_cursor = if app.cursor_visible {
                let mut chars: Vec<char> = app.commit_message.chars().collect();
                chars.insert(app.cursor_position, '_');
                chars.into_iter().collect()
            } else {
                app.commit_message.clone()
            };
            let input = Paragraph::new(message_with_cursor)
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Commit Message"),
                );

            let help = Paragraph::new("Type your commit message, Enter to confirm, Esc to quit")
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
                .split(chunks[1]);

            f.render_widget(input, layout[0]);
            f.render_widget(help, layout[1]);
        }
        AppState::BranchPrefixSelection => {
            let items: Vec<ListItem> = BRANCH_PREFIXES
                .iter()
                .enumerate()
                .map(|(i, prefix)| {
                    let style = if i == app.selected_branch_prefix_index {
                        Style::default().bg(Color::DarkGray).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    ListItem::new(*prefix).style(style)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title("Select Branch Prefix")
                    .borders(Borders::ALL),
            );

            let help = Paragraph::new("Use ↑↓ to navigate, Enter to select, Esc to quit")
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                .split(chunks[1]);

            f.render_widget(list, layout[0]);
            f.render_widget(help, layout[1]);
        }
        AppState::BranchStoryInput => {
            let story_with_cursor = if app.cursor_visible {
                let mut chars: Vec<char> = app.branch_story.chars().collect();
                chars.insert(app.cursor_position, '_');
                chars.into_iter().collect()
            } else {
                app.branch_story.clone()
            };
            let input = Paragraph::new(story_with_cursor)
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Jira Story Number (optional, numbers only)"),
                );

            let help = Paragraph::new("Enter story number or press Enter to skip, Esc to quit")
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
                .split(chunks[1]);

            f.render_widget(input, layout[0]);
            f.render_widget(help, layout[1]);
        }
        AppState::BranchNameInput => {
            let name_with_cursor = if app.cursor_visible {
                let mut chars: Vec<char> = app.branch_name.chars().collect();
                chars.insert(app.cursor_position, '_');
                chars.into_iter().collect()
            } else {
                app.branch_name.clone()
            };
            let input = Paragraph::new(name_with_cursor)
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Branch Name"));

            let help = Paragraph::new("Enter branch name, Enter to create branch, Esc to quit")
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
                .split(chunks[1]);

            f.render_widget(input, layout[0]);
            f.render_widget(help, layout[1]);
        }
    }
}
