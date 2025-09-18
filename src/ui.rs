use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::{App, AppState, COMMIT_PREFIXES};

/// Renders the TUI interface based on current application state
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area());

    let title = Paragraph::new("Git Commit Creator (gitcc) 🚀")
        .style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    match app.state {
        AppState::StagedFilesReview => {
            let items: Vec<ListItem> = app
                .staged_files
                .iter()
                .map(|f| ListItem::new(format!("• {f}")))
                .collect();

            let files_list = List::new(items)
                .block(
                    Block::default()
                        .title("🗂️ Staged Files")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::Cyan));

            let help = Paragraph::new("Press 'y' to proceed, 'n' to abort")
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });

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
    }
}
