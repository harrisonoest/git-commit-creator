use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
    Frame,
};

use crate::{App, AppState};

/// Renders the TUI interface based on current application state
pub fn render(f: &mut Frame, app: &mut App) {
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
                    let status_indicator =
                        app.file_statuses.get(f).map(|s| s.as_str()).unwrap_or("?");
                    let prefix = if is_staged { "[S]" } else { "[ ]" };
                    let style = if i == app.selected_file_index {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else if is_staged {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };
                    ListItem::new(format!("{prefix} [{status_indicator}] {f}")).style(style)
                })
                .collect();

            let files_list = List::new(items)
                .block(Block::default().title("📁 Files").borders(Borders::ALL))
                .style(Style::default());

            // Format diff with color coding
            let all_diff_lines: Vec<ratatui::text::Line> = app
                .current_diff
                .lines()
                .map(|line| {
                    let style = if line.starts_with('+') && !line.starts_with("+++") {
                        Style::default().fg(Color::Green)
                    } else if line.starts_with('-') && !line.starts_with("---") {
                        Style::default().fg(Color::Red)
                    } else if line.starts_with("@@") {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    };
                    ratatui::text::Line::from(ratatui::text::Span::styled(line, style))
                })
                .collect();

            // Apply scroll offset
            let diff_lines: Vec<ratatui::text::Line> = all_diff_lines
                .into_iter()
                .skip(app.diff_scroll_offset)
                .collect();

            let selected_file = app
                .all_files
                .get(app.selected_file_index)
                .map(|s| s.as_str())
                .unwrap_or("");

            let total_lines = app.current_diff.lines().count();
            let scroll_indicator = if total_lines > 0 && app.diff_scroll_offset > 0 {
                format!(" (line {}/{})", app.diff_scroll_offset + 1, total_lines)
            } else {
                String::new()
            };

            let diff_widget = Paragraph::new(diff_lines)
                .block(
                    Block::default()
                        .title(format!("📝 Diff: {}{}", selected_file, scroll_indicator))
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: false });

            let help = if app.staged_files_set.is_empty() {
                Paragraph::new("⚠️ No files staged - stage at least one file to proceed")
                    .style(Style::default().fg(Color::Red))
                    .wrap(Wrap { trim: true })
            } else {
                Paragraph::new(
                    "↑↓ - scroll files, j/k - scroll diff, Space - stage, Enter - proceed, Esc - abort",
                )
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true })
            };

            // Calculate file list size: number of files + 2 for borders, minimum 3 lines
            let file_list_size = (app.all_files.len() + 2).max(3) as u16;

            // Check if we have enough space for diff (need at least 10 lines total)
            let available_height = chunks[1].height;
            let show_diff = available_height >= 10;

            let layout = if show_diff {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(file_list_size),
                        Constraint::Min(5),
                        Constraint::Length(3),
                    ])
                    .split(chunks[1])
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(3), Constraint::Length(3)])
                    .split(chunks[1])
            };

            f.render_widget(files_list, layout[0]);

            if show_diff {
                // Update visible lines for scroll calculation (subtract 2 for borders)
                app.diff_visible_lines = layout[1].height.saturating_sub(2) as usize;

                f.render_widget(diff_widget, layout[1]);

                // Render scrollbar for diff area
                let total_lines = app.current_diff.lines().count();
                if total_lines > app.diff_visible_lines {
                    let max_scroll = total_lines.saturating_sub(app.diff_visible_lines);
                    let mut scrollbar_state = ScrollbarState::default()
                        .content_length(max_scroll.saturating_add(1))
                        .position(app.diff_scroll_offset);

                    let scrollbar = Scrollbar::default()
                        .orientation(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("↑"))
                        .end_symbol(Some("↓"));

                    f.render_stateful_widget(scrollbar, layout[1], &mut scrollbar_state);
                }

                f.render_widget(help, layout[2]);
            } else {
                f.render_widget(help, layout[1]);
            }
        }
        AppState::PrefixSelection => {
            let filtered_prefixes = app.filtered_commit_prefixes();
            let items: Vec<ListItem> = filtered_prefixes
                .iter()
                .enumerate()
                .map(|(i, prefix)| {
                    let style = if i == app.selected_prefix_index {
                        Style::default().bg(Color::DarkGray).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    ListItem::new(prefix.as_str()).style(style)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title("Select Commit Prefix")
                    .borders(Borders::ALL),
            );

            let filter_display = if app.filter.is_empty() {
                "Type to filter...".to_string()
            } else {
                app.filter.to_string()
            };

            let filter_widget = Paragraph::new(filter_display)
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("Filter"));

            let help = Paragraph::new("Type to filter, ↑↓ to navigate, Enter to select, Backspace to clear filter, Esc to quit")
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(0),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            f.render_widget(list, layout[0]);
            f.render_widget(filter_widget, layout[1]);
            f.render_widget(help, layout[2]);
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
            let filtered_prefixes = app.filtered_branch_prefixes();
            let items: Vec<ListItem> = filtered_prefixes
                .iter()
                .enumerate()
                .map(|(i, prefix)| {
                    let style = if i == app.selected_branch_prefix_index {
                        Style::default().bg(Color::DarkGray).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    ListItem::new(prefix.as_str()).style(style)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title("Select Branch Prefix")
                    .borders(Borders::ALL),
            );

            let filter_display = if app.filter.is_empty() {
                "Type to filter...".to_string()
            } else {
                format!("Filter: {}", app.filter)
            };

            let filter_widget = Paragraph::new(filter_display)
                .style(Style::default().fg(Color::Cyan))
                .block(Block::default().borders(Borders::ALL).title("Filter"));

            let help = Paragraph::new("Type to filter, ↑↓ to navigate, Enter to select, Backspace to clear filter, Esc to quit")
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(0),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            f.render_widget(list, layout[0]);
            f.render_widget(filter_widget, layout[1]);
            f.render_widget(help, layout[2]);
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
