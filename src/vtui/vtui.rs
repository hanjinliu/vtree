use std::io::Write;
use tui::{
    backend::Backend,
    widgets::{Block, Borders, Paragraph},
    // layout::{Layout, Constraint, Direction},
    style::{Color, Style},
    text::Text,
    Terminal,
    Frame,
};
use crossterm::{
    event::{self, Event, KeyEvent, KeyCode, KeyModifiers},
};
use super::{
    rich::{RichText, RichLine},
    history::History,
    super::{
        terminal::parse_string_with_quote,
        tree,
    },
};

const _VIRTUAL_FILES: &str = "virtual-files";

pub struct App {
    pub lines: History<RichLine>,
    pub buffer: String,
    pub cursor_pos: usize,
    pub selection: Option<(usize, usize)>,
    pub tree: tree::TreeModel,
    pub history: History<String>,
}

impl App {
    pub fn new(tree: tree::TreeModel) -> App {
        App {
            lines: History::new(1000),
            buffer: String::new(),
            cursor_pos: 0,
            selection: None,
            tree,
            history: History::new(500),
        }
    }
    
    pub fn flush_buffer(&mut self) {
        let idx = self.lines.len() - 1;
        for p in self.rich_buffer() {
            self.lines[idx].push(p);
        }
        self.clear_buffer();
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
        self.cursor_pos = 0;
    }

    pub fn run_buffer(&mut self) {
        let buf = self.buffer.clone();
        self.history.add(buf);
        self.flush_buffer();
    }

    pub fn set_buffer(&mut self, buf: String) {
        self.buffer = buf;
        self.cursor_pos = self.buffer.len();
    }
    
    pub fn print_text(&mut self, s: String) {
        s.split("\n").for_each(|s| {
            let mut line = RichLine::new();
            line.push(RichText::new(s.to_string(), Color::White));
            self.lines.add(line);
        });
    }

    pub fn print_error<E: std::error::Error>(&mut self, e: E) {
        let text = format!("{}", e);
        text.split("\n").for_each(|s| {
            let mut line = RichLine::new();
            line.push(RichText::new(s.to_string(), Color::Red));
            self.lines.add(line);
        });
    }

    /// Get the vector of RichTexts from the buffer.
    pub fn rich_buffer(&self) -> Vec<RichText> {
        let strs = parse_string_with_quote(&self.buffer);
        let nstr = strs.len();
        if nstr == 0 {
            return Vec::new();
        }
        else {
            let cmd = RichText::new(
                strs.get(0).unwrap().to_string(), 
                Color::Yellow
            );
            let mut args = Vec::new();
            args.push(cmd);
            for str in strs[1..].iter() {
                if str.starts_with("\"") || str.starts_with("\'") {
                    args.push(RichText::new(" ".to_string() + str, Color::Blue));
                }
                else {
                    args.push(RichText::new(" ".to_string() + str, Color::White));
                }
            }
            return args;
        }
    }

    pub fn get_text(&self) -> Text {
        let mut text = Text::from("");
        let nlines = self.lines.len();
        let mut iter = self.lines.iter();

        if nlines == 0 {
            return text;
        }
        else if nlines > 1 {
            for _ in 0..nlines - 1 {
                let line = iter.next().unwrap();
                text.extend([line.as_spans()]);
            }
        }
        let mut last_line = iter.next().unwrap().clone();
        let rbuf = self.rich_buffer();
        last_line.extend(rbuf);
        text.extend([last_line.as_spans()]);
        text
    }
}

pub fn process_keys<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> std::io::Result<String> {
    let _ = std::io::stdout().flush();  // flush stdout
    let prefix = app.tree.as_prefix();
    app.print_text(prefix);
    let output = loop {
        terminal.draw(|f| ui(f, &app))?;
        if let Event::Key(KeyEvent {code, modifiers, ..}) = event::read()? {
            match (code, modifiers) {
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    let output = app.buffer.clone();
                    app.buffer.push('\n');
                    app.run_buffer();
                    break output;
                },
                (KeyCode::Backspace, _) => {app.buffer.pop();},
                (KeyCode::Esc, _) => {app.clear_buffer();},
                (KeyCode::Left, KeyModifiers::NONE) => {
                    if app.cursor_pos > 0 {
                        app.cursor_pos -= 1;
                    }
                },
                (KeyCode::Right, KeyModifiers::NONE) => {
                    if app.cursor_pos <= app.buffer.len() {
                        app.cursor_pos += 1;
                    }
                },
                (KeyCode::Char(c), KeyModifiers::NONE) => app.buffer.push(c),
                (KeyCode::Char(c), KeyModifiers::SHIFT) => app.buffer.push(c),
                (KeyCode::Char(c), KeyModifiers::CONTROL) => {
                    match c {
                        'c' => {println!("Ctrl+C")},
                        _ => {},
                    }
                },
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
                _ => {},
            }
        }
    };
    Ok(output)
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let rect = f.size();
    // let chunks = Layout::default().direction(Direction::Vertical).margin(1).split(rect);
    let input = Paragraph::new(app.get_text())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("VTree"));
    
    f.render_widget(input, rect);
    // f.set_cursor(
    //     chunks[0].x + app.prefix.width() as u16 + 1,
    //     chunks[0].y + 1,
    // )
}
