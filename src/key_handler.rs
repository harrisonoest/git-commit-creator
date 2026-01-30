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

/// Common text input handling result
enum TextInputResult {
    Handled,
    Submit,
    Cancel,
    Unhandled,
}

/// Handles common text input operations (cursor movement, deletion, etc.)
fn handle_text_input(
    text: &mut String,
    cursor_pos: &mut usize,
    key: KeyCode,
    modifiers: KeyModifiers,
    allow_word_ops: bool,
) -> TextInputResult {
    match key {
        KeyCode::Enter => TextInputResult::Submit,
        KeyCode::Esc => TextInputResult::Cancel,
        KeyCode::Char(c) => {
            if allow_word_ops && modifiers.contains(KeyModifiers::ALT) && c == 'd' {
                delete_word_forward(text, cursor_pos);
            } else {
                text.insert(*cursor_pos, c);
                *cursor_pos += 1;
            }
            TextInputResult::Handled
        }
        KeyCode::Backspace => {
            if allow_word_ops
                && (modifiers.contains(KeyModifiers::CONTROL)
                    || modifiers.contains(KeyModifiers::ALT))
            {
                delete_word_backward(text, cursor_pos);
            } else if *cursor_pos > 0 {
                *cursor_pos -= 1;
                text.remove(*cursor_pos);
            }
            TextInputResult::Handled
        }
        KeyCode::Delete => {
            if allow_word_ops && modifiers.contains(KeyModifiers::CONTROL) {
                delete_word_forward(text, cursor_pos);
            } else if *cursor_pos < text.len() {
                text.remove(*cursor_pos);
            }
            TextInputResult::Handled
        }
        KeyCode::Left => {
            if allow_word_ops
                && (modifiers.contains(KeyModifiers::CONTROL)
                    || modifiers.contains(KeyModifiers::ALT))
            {
                *cursor_pos = find_prev_word(text, *cursor_pos);
            } else if *cursor_pos > 0 {
                *cursor_pos -= 1;
            }
            TextInputResult::Handled
        }
        KeyCode::Right => {
            if allow_word_ops
                && (modifiers.contains(KeyModifiers::CONTROL)
                    || modifiers.contains(KeyModifiers::ALT))
            {
                *cursor_pos = find_next_word(text, *cursor_pos);
            } else if *cursor_pos < text.len() {
                *cursor_pos += 1;
            }
            TextInputResult::Handled
        }
        KeyCode::Home => {
            *cursor_pos = 0;
            TextInputResult::Handled
        }
        KeyCode::End => {
            *cursor_pos = text.len();
            TextInputResult::Handled
        }
        _ => TextInputResult::Unhandled,
    }
}

/// Handles keyboard input based on current application state
pub fn handle_key(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match app.state {
        AppState::StagedFilesReview => match key {
            KeyCode::Up => {
                if app.selected_file_index > 0 {
                    app.selected_file_index -= 1;
                } else {
                    app.selected_file_index = app.all_files.len().saturating_sub(1);
                }
                app.update_current_diff();
            }
            KeyCode::Down => {
                if app.selected_file_index < app.all_files.len().saturating_sub(1) {
                    app.selected_file_index += 1;
                } else {
                    app.selected_file_index = 0;
                }
                app.update_current_diff();
            }
            KeyCode::Char('j') => {
                let total_lines = app.current_diff.lines().count();
                let max_scroll = total_lines.saturating_sub(app.diff_visible_lines);
                app.diff_scroll_offset = (app.diff_scroll_offset + 1).min(max_scroll);
            }
            KeyCode::Char('k') => {
                app.diff_scroll_offset = app.diff_scroll_offset.saturating_sub(1);
            }
            KeyCode::Char(' ') => {
                if let Some(file) = app.all_files.get(app.selected_file_index) {
                    if app.staged_files_set.contains(file) {
                        let _ = crate::git::unstage_file(file);
                        app.staged_files_set.remove(file);
                    } else {
                        let _ = crate::git::stage_file(file);
                        app.staged_files_set.insert(file.clone());
                    }
                    app.update_current_diff();
                }
            }
            KeyCode::Enter => {
                if app.staged_files_set.is_empty() {
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
            KeyCode::Esc => app.should_quit = true,
            _ => {}
        },
        AppState::PrefixSelection => {
            handle_prefix_selection(app, key, App::filtered_commit_prefixes, |app, prefix| {
                app.prefix = Some(prefix);
                app.selected_prefix_index = 0;
                if app.message.is_some() {
                    app.should_quit = true;
                } else {
                    app.state = AppState::MessageInput;
                }
            });
        }
        AppState::MessageInput => {
            match handle_text_input(
                &mut app.commit_message,
                &mut app.cursor_position,
                key,
                modifiers,
                true,
            ) {
                TextInputResult::Submit => {
                    if !app.commit_message.trim().is_empty() {
                        app.message = Some(app.commit_message.clone());
                        app.should_quit = true;
                    }
                }
                TextInputResult::Cancel => app.should_quit = true,
                _ => {}
            }
        }
        AppState::BranchPrefixSelection => {
            handle_prefix_selection(app, key, App::filtered_branch_prefixes, |app, prefix| {
                app.branch_prefix = Some(prefix);
                app.state = AppState::BranchStoryInput;
                app.cursor_position = app.branch_story.len();
            });
        }
        AppState::BranchStoryInput => {
            // Only allow digits for story input
            if let KeyCode::Char(c) = key {
                if !c.is_ascii_digit() {
                    return;
                }
            }
            match handle_text_input(
                &mut app.branch_story,
                &mut app.cursor_position,
                key,
                modifiers,
                false,
            ) {
                TextInputResult::Submit => {
                    app.state = AppState::BranchNameInput;
                    app.cursor_position = app.branch_name.len();
                }
                TextInputResult::Cancel => app.should_quit = true,
                _ => {}
            }
        }
        AppState::BranchNameInput => {
            match handle_text_input(
                &mut app.branch_name,
                &mut app.cursor_position,
                key,
                modifiers,
                false,
            ) {
                TextInputResult::Submit => {
                    if !app.branch_name.trim().is_empty() {
                        app.should_proceed = true;
                        app.should_quit = true;
                    }
                }
                TextInputResult::Cancel => app.should_quit = true,
                _ => {}
            }
        }
        AppState::BranchSearch => {
            if app.matching_branches.is_empty() {
                match handle_text_input(
                    &mut app.search_query,
                    &mut app.cursor_position,
                    key,
                    modifiers,
                    false,
                ) {
                    TextInputResult::Submit => {
                        if !app.search_query.trim().is_empty() {
                            app.should_proceed = true;
                        }
                    }
                    TextInputResult::Cancel => app.should_quit = true,
                    _ => {}
                }
            } else {
                match key {
                    KeyCode::Up => {
                        if app.selected_branch_index > 0 {
                            app.selected_branch_index -= 1;
                            if app.selected_branch_index < app.branch_scroll_offset {
                                app.branch_scroll_offset = app.selected_branch_index;
                            }
                        }
                    }
                    KeyCode::Down => {
                        if app.selected_branch_index < app.matching_branches.len() - 1 {
                            app.selected_branch_index += 1;
                            let max_visible = app.branch_scroll_offset + app.branch_visible_lines;
                            if app.selected_branch_index >= max_visible {
                                app.branch_scroll_offset = app.selected_branch_index.saturating_sub(app.branch_visible_lines - 1);
                            }
                        }
                    }
                    KeyCode::Enter => {
                        app.should_proceed = true;
                        app.should_quit = true;
                    }
                    KeyCode::Esc => app.should_quit = true,
                    _ => {}
                }
            }
        }
    }
}

/// Generic prefix selection handler
fn handle_prefix_selection<F, G>(app: &mut App, key: KeyCode, get_filtered: F, on_select: G)
where
    F: Fn(&App) -> Vec<String>,
    G: FnOnce(&mut App, String),
{
    let filtered = get_filtered(app);
    let index = if matches!(app.state, AppState::BranchPrefixSelection) {
        &mut app.selected_branch_prefix_index
    } else {
        &mut app.selected_prefix_index
    };

    match key {
        KeyCode::Up => {
            if *index > 0 {
                *index -= 1;
            } else {
                *index = filtered.len().saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if *index < filtered.len().saturating_sub(1) {
                *index += 1;
            } else {
                *index = 0;
            }
        }
        KeyCode::Enter => {
            if !filtered.is_empty() && *index < filtered.len() {
                let selected = filtered[*index].clone();
                on_select(app, selected);
            }
        }
        KeyCode::Char(c) => {
            app.filter.push(c);
            *index = 0;
        }
        KeyCode::Backspace => {
            if !app.filter.is_empty() {
                app.filter.pop();
                *index = 0;
            }
        }
        KeyCode::Esc => app.should_quit = true,
        _ => {}
    }
}
