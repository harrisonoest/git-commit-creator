use crossterm::event::{KeyCode, KeyModifiers};

use crate::{App, AppState};

/// Deletes word backward from cursor position
fn delete_word_backward(text: &mut String, cursor_pos: &mut usize) {
    let chars: Vec<char> = text.chars().collect();
    let start_pos = find_prev_word(text, *cursor_pos);

    if start_pos < *cursor_pos {
        let mut new_chars = chars;
        new_chars.drain(start_pos..*cursor_pos);
        *text = new_chars.into_iter().collect();
        *cursor_pos = start_pos;
    }
}

/// Deletes word forward from cursor position
fn delete_word_forward(text: &mut String, cursor_pos: &mut usize) {
    let chars: Vec<char> = text.chars().collect();
    let mut pos = *cursor_pos;

    // Skip current whitespace
    while pos < chars.len() && chars[pos] == ' ' {
        pos += 1;
    }

    // Delete word characters
    let start_pos = pos;
    while pos < chars.len() && chars[pos] != ' ' {
        pos += 1;
    }

    if start_pos < pos {
        let mut new_chars = chars;
        new_chars.drain(start_pos..pos);
        *text = new_chars.into_iter().collect();
    }
}

/// Finds next word boundary
fn find_next_word(text: &str, cursor_pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut pos = cursor_pos;

    // Skip current word
    while pos < chars.len() && chars[pos] != ' ' {
        pos += 1;
    }

    // Skip whitespace
    while pos < chars.len() && chars[pos] == ' ' {
        pos += 1;
    }

    pos
}

/// Finds previous word boundary
fn find_prev_word(text: &str, cursor_pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    if cursor_pos == 0 {
        return 0;
    }

    let mut pos = cursor_pos.saturating_sub(1);

    // Skip current whitespace
    while pos > 0 && chars[pos] == ' ' {
        pos = pos.saturating_sub(1);
    }

    // Skip to start of word
    while pos > 0 && chars[pos.saturating_sub(1)] != ' ' {
        pos = pos.saturating_sub(1);
    }

    pos
}

/// Handles keyboard input based on current application state
pub fn handle_key(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match app.state {
        AppState::StagedFilesReview => match key {
            KeyCode::Up => {
                if app.selected_file_index > 0 {
                    app.selected_file_index -= 1;
                }
            }
            KeyCode::Down => {
                if app.selected_file_index < app.all_files.len().saturating_sub(1) {
                    app.selected_file_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(file) = app.all_files.get(app.selected_file_index) {
                    if app.staged_files_set.contains(file) {
                        let _ = crate::git::unstage_file(file);
                        app.staged_files_set.remove(file);
                    } else {
                        let _ = crate::git::stage_file(file);
                        app.staged_files_set.insert(file.clone());
                    }
                }
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if app.staged_files_set.is_empty() {
                    // Don't proceed if no files are staged
                    return;
                }
                app.should_proceed = true;
                if app.prefix.is_some() && app.message.is_some() {
                    app.should_quit = true;
                } else if app.prefix.is_some() {
                    app.state = AppState::MessageInput;
                } else {
                    app.filter.clear();
                    app.selected_prefix_index = 0;
                    app.state = AppState::PrefixSelection;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.should_quit = true;
            }
            _ => {}
        },
        AppState::PrefixSelection => match key {
            KeyCode::Up => {
                let filtered = app.filtered_commit_prefixes();
                if app.selected_prefix_index > 0 {
                    app.selected_prefix_index -= 1;
                } else {
                    app.selected_prefix_index = filtered.len().saturating_sub(1);
                }
            }
            KeyCode::Down => {
                let filtered = app.filtered_commit_prefixes();
                if app.selected_prefix_index < filtered.len().saturating_sub(1) {
                    app.selected_prefix_index += 1;
                } else {
                    app.selected_prefix_index = 0;
                }
            }
            KeyCode::Enter => {
                let filtered = app.filtered_commit_prefixes();
                if !filtered.is_empty() && app.selected_prefix_index < filtered.len() {
                    let selected_prefix = &filtered[app.selected_prefix_index];
                    app.prefix = Some(selected_prefix.clone());

                    if app.message.is_some() {
                        app.should_quit = true;
                    } else {
                        app.state = AppState::MessageInput;
                    }
                }
            }
            KeyCode::Char(c) => {
                app.filter.push(c);
                app.selected_prefix_index = 0;
            }
            KeyCode::Backspace => {
                if !app.filter.is_empty() {
                    app.filter.pop();
                    app.selected_prefix_index = 0;
                }
            }
            KeyCode::Esc => app.should_quit = true,
            _ => {}
        },
        AppState::MessageInput => match key {
            KeyCode::Enter => {
                if !app.commit_message.trim().is_empty() {
                    app.message = Some(app.commit_message.clone());
                    app.should_quit = true;
                }
            }
            KeyCode::Char(c) => {
                if modifiers.contains(KeyModifiers::ALT) && c == 'd' {
                    delete_word_forward(&mut app.commit_message, &mut app.cursor_position);
                } else {
                    app.commit_message.insert(app.cursor_position, c);
                    app.cursor_position += 1;
                }
            }
            KeyCode::Backspace => {
                if modifiers.contains(KeyModifiers::CONTROL)
                    || modifiers.contains(KeyModifiers::ALT)
                {
                    delete_word_backward(&mut app.commit_message, &mut app.cursor_position);
                } else if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                    app.commit_message.remove(app.cursor_position);
                }
            }
            KeyCode::Delete => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    delete_word_forward(&mut app.commit_message, &mut app.cursor_position);
                } else if app.cursor_position < app.commit_message.len() {
                    app.commit_message.remove(app.cursor_position);
                }
            }
            KeyCode::Left => {
                if modifiers.contains(KeyModifiers::CONTROL)
                    || modifiers.contains(KeyModifiers::ALT)
                {
                    app.cursor_position = find_prev_word(&app.commit_message, app.cursor_position);
                } else if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if modifiers.contains(KeyModifiers::CONTROL)
                    || modifiers.contains(KeyModifiers::ALT)
                {
                    app.cursor_position = find_next_word(&app.commit_message, app.cursor_position);
                } else if app.cursor_position < app.commit_message.len() {
                    app.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                app.cursor_position = 0;
            }
            KeyCode::End => {
                app.cursor_position = app.commit_message.len();
            }
            KeyCode::Esc => app.should_quit = true,
            _ => {}
        },
        AppState::BranchPrefixSelection => match key {
            KeyCode::Up => {
                let filtered = app.filtered_branch_prefixes();
                if app.selected_branch_prefix_index > 0 {
                    app.selected_branch_prefix_index -= 1;
                } else {
                    app.selected_branch_prefix_index = filtered.len().saturating_sub(1);
                }
            }
            KeyCode::Down => {
                let filtered = app.filtered_branch_prefixes();
                if app.selected_branch_prefix_index < filtered.len().saturating_sub(1) {
                    app.selected_branch_prefix_index += 1;
                } else {
                    app.selected_branch_prefix_index = 0;
                }
            }
            KeyCode::Enter => {
                let filtered = app.filtered_branch_prefixes();
                if !filtered.is_empty() && app.selected_branch_prefix_index < filtered.len() {
                    let selected_prefix = &filtered[app.selected_branch_prefix_index];
                    app.branch_prefix = Some(selected_prefix.clone());
                    app.state = AppState::BranchStoryInput;
                    app.cursor_position = app.branch_story.len();
                }
            }
            KeyCode::Char(c) => {
                app.filter.push(c);
                app.selected_branch_prefix_index = 0;
            }
            KeyCode::Backspace => {
                if !app.filter.is_empty() {
                    app.filter.pop();
                    app.selected_branch_prefix_index = 0;
                }
            }
            KeyCode::Esc => app.should_quit = true,
            _ => {}
        },
        AppState::BranchStoryInput => match key {
            KeyCode::Enter => {
                app.state = AppState::BranchNameInput;
                app.cursor_position = app.branch_name.len();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                app.branch_story.insert(app.cursor_position, c);
                app.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                    app.branch_story.remove(app.cursor_position);
                }
            }
            KeyCode::Delete => {
                if app.cursor_position < app.branch_story.len() {
                    app.branch_story.remove(app.cursor_position);
                }
            }
            KeyCode::Left => {
                if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if app.cursor_position < app.branch_story.len() {
                    app.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                app.cursor_position = 0;
            }
            KeyCode::End => {
                app.cursor_position = app.branch_story.len();
            }
            KeyCode::Esc => app.should_quit = true,
            _ => {}
        },
        AppState::BranchNameInput => match key {
            KeyCode::Enter => {
                if !app.branch_name.trim().is_empty() {
                    app.should_proceed = true;
                    app.should_quit = true;
                }
            }
            KeyCode::Char(c) => {
                app.branch_name.insert(app.cursor_position, c);
                app.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                    app.branch_name.remove(app.cursor_position);
                }
            }
            KeyCode::Delete => {
                if app.cursor_position < app.branch_name.len() {
                    app.branch_name.remove(app.cursor_position);
                }
            }
            KeyCode::Left => {
                if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if app.cursor_position < app.branch_name.len() {
                    app.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                app.cursor_position = 0;
            }
            KeyCode::End => {
                app.cursor_position = app.branch_name.len();
            }
            KeyCode::Esc => app.should_quit = true,
            _ => {}
        },
    }
}
