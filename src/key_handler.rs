use crossterm::event::{KeyCode, KeyModifiers};

use crate::{App, AppState, COMMIT_PREFIXES};

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
                if app.selected_prefix_index > 0 {
                    app.selected_prefix_index -= 1;
                }
            }
            KeyCode::Down => {
                if app.selected_prefix_index < COMMIT_PREFIXES.len() - 1 {
                    app.selected_prefix_index += 1;
                }
            }
            KeyCode::Enter => {
                let selected_prefix = COMMIT_PREFIXES[app.selected_prefix_index];
                app.prefix = Some(selected_prefix.to_string());

                if app.message.is_some() {
                    app.should_quit = true;
                } else {
                    app.state = AppState::MessageInput;
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
    }
}
