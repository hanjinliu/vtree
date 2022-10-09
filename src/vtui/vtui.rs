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
    pub scroll_pos: usize,
}

impl App {
    pub fn new(tree: tree::TreeModel) -> Self {
        Self {
            lines: History::new(1000),
            buffer: String::new(),
            cursor_pos: 0,
            selection: None,
            tree,
            history: History::new(500),
            scroll_pos: 0,
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
        self.scroll_pos = 0;
    }

    pub fn print_error<E: std::error::Error>(&mut self, e: E) {
        let text = format!("{}", e);
        text.split("\n").for_each(|s| {
            let mut line = RichLine::new();
            line.push(RichText::new(s.to_string(), Color::Red));
            self.lines.add(line);
        });
        self.scroll_pos = 0
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

    pub fn get_text(&self, nlines: usize) -> Text {
        let mut text = Text::from("");
        let nlines_total = self.lines.len();
        if nlines_total <= self.scroll_pos || nlines == 0 {
            return text;
        }
        let stop = nlines_total - self.scroll_pos;
        let (start, nlines) = if stop >= nlines {
            (stop - nlines, nlines)
        } else {
            (0, stop)
        };
        let mut iter = self.lines.iter().skip(start);
        if nlines > 1 {
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
        terminal.draw(|f| render_ui(f, app))?;
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
