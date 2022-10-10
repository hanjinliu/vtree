use std::io::Write;
use tui::{
    backend::Backend,
    widgets::{Block, Borders, Paragraph},
    style::{Color, Style},
    Terminal,
    Frame,
};
use crossterm::{
    event::{self, Event, KeyEvent, KeyCode, KeyModifiers},
};
use super::app::App;

mod clipboard {
    use arboard::Clipboard;

    pub fn get_text() -> String {
        match Clipboard::new() {
            Ok(mut clip) => clip.get_text().unwrap_or("".to_string()),
            Err(_) => "".to_string()
        }
    }
    
    pub fn set_text(text: &String) {
        match Clipboard::new() {
            Ok(mut clip) => clip.set_text(text).unwrap_or(()),
            Err(_) => {}
        }
    }
}
const _VIRTUAL_FILES: &str = "virtual-files";


pub fn process_keys<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> std::io::Result<String> {
    let _ = std::io::stdout().flush();  // flush stdout
    let prefix = app.tree.as_prefix();
    app.print_text(prefix);
    let output = loop {
        terminal.draw(|f| render_ui(f, app))?;
        if let Event::Key(KeyEvent {code, modifiers, ..}) = event::read()? {
            match (code, modifiers) {
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    let output = app.buffer.clone();
                    app.buffer.push('\n');
                    app.run_buffer();
                    break output;
                },
                (KeyCode::Backspace, KeyModifiers::NONE) => { app.text_backspace_event(); },
                (KeyCode::Delete, KeyModifiers::NONE) => { app.text_delete_event(); },
                (KeyCode::Tab, KeyModifiers::NONE) => {}, // TODO: tab completion
                (KeyCode::Esc, KeyModifiers::NONE) => {app.clear_buffer();},
                (KeyCode::Left, KeyModifiers::NONE) => {
                    app.text_move_cursor(-1, false) 
                },
                (KeyCode::Right, KeyModifiers::NONE) => { 
                    app.text_move_cursor(1, false) 
                },
                (KeyCode::Left, KeyModifiers::CONTROL) => { 
                    app.text_move_cursor_to_prev_word(false);
                },
                (KeyCode::Right, KeyModifiers::CONTROL) => {
                    app.text_move_cursor_to_next_word(false)
                },
                (KeyCode::Left, KeyModifiers::SHIFT) => {
                    app.text_move_cursor(-1, true)
                },
                (KeyCode::Right, KeyModifiers::SHIFT) => {
                    app.text_move_cursor(1, true)
                },
                (KeyCode::Left, _) => {
                    if modifiers.contains(KeyModifiers::CONTROL) && modifiers.contains(KeyModifiers::SHIFT) {
                        app.text_move_cursor_to_prev_word(true);
                    };
                },
                (KeyCode::Right, _) => {
                    if modifiers.contains(KeyModifiers::CONTROL) && modifiers.contains(KeyModifiers::SHIFT) {
                        app.text_move_cursor_to_next_word(true);
                    };
                },
                (KeyCode::Home, KeyModifiers::NONE) => { app.cursor.move_to(0) },
                (KeyCode::End, KeyModifiers::NONE) => { app.cursor.move_to(app.buffer.len()) },
                (KeyCode::Home, KeyModifiers::SHIFT) => { app.cursor.select_to(0) },
                (KeyCode::End, KeyModifiers::SHIFT) => { app.cursor.select_to(app.buffer.len()) },
                (KeyCode::Char(c), KeyModifiers::NONE) => app.text_add_char(c),
                (KeyCode::Char(c), KeyModifiers::SHIFT) => app.text_add_char(c),
                (KeyCode::Char(c), KeyModifiers::CONTROL) => {
                    match c {
                        'a' => {
                            app.cursor.move_to(0);
                            app.cursor.select_to(app.buffer.len());
                        },
                        'c' => {
                            let text = if app.cursor.selection_size() > 0 {
                                app.text_selected()
                            } else {
                                app.buffer.clone()
                            };
                            clipboard::set_text(&text);
                        },
                        'v' => {
                            app.insert_text(clipboard::get_text());
                        },
                        'x' => {
                            if app.cursor.selection_size() > 0 {
                                let text = app.text_selected();
                                clipboard::set_text(&text);
                                app.text_backspace_event();
                            } else {
                                let text = app.buffer.clone();
                                clipboard::set_text(&text);
                                app.clear_buffer();
                            }
                        },
                        _ => {},
                    }
                },
                // browse history
                (KeyCode::Up, KeyModifiers::NONE) => {
                    if let Some(buf) = app.history.prev() {
                        app.set_buffer(buf);
                    }
                },
                (KeyCode::Down, KeyModifiers::NONE) => {
                    match app.history.next() {
                        Some(buf) => app.set_buffer(buf),
                        None => app.clear_buffer(),
                    }
                },
                // scroll
                (KeyCode::Up, KeyModifiers::SHIFT) => {
                    if app.scroll_pos < app.lines.len() {
                        app.scroll_pos += 1;
                    }
                },
                (KeyCode::Down, KeyModifiers::SHIFT) => {
                    if app.scroll_pos > 0 {
                        app.scroll_pos -= 1;
                    }
                },
                _ => {},
            }
        }
    };
    Ok(output)
}

fn render_ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let rect = f.size();
    // let chunks = Layout::default().direction(Direction::Vertical).margin(1).split(rect);

    // NOTE: height of text area is 2 less than the height of the terminal (two borders).
    let h = rect.height as usize - 2;
    if app.lines.len() >= h {
        let max_scroll_pos = app.lines.len() - h;
        if app.scroll_pos > max_scroll_pos {
            app.scroll_pos = max_scroll_pos;
        }
    }

    let input = Paragraph::new(app.get_text(h))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL)
        .title("VTree"));
    
    f.render_widget(input, rect);
    
    // f.set_cursor(
    //     chunks[0].x + app.prefix.width() as u16 + 1,
    //     chunks[0].y + 1,
    // )
}
