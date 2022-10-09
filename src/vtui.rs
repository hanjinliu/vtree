use std::{io, io::Write};
use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Widget, Block, Borders, Paragraph},
    // layout::{Layout, Constraint, Direction},
    style::{Color, Style},
    text::{Span, Spans, Text},
    Terminal,
    Frame,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::path::PathBuf;
use super::terminal::{VCommand, parse_string_with_quote};
use super::tree;
use super::{get_json_path, get_relative_vtree_path};

const _VIRTUAL_FILES: &str = "virtual-files";

#[derive(Debug, Clone)]
struct RichText {
    text: String,
    style: Style,
}

impl RichText {
    fn new(text: String, color: Color) -> Self {
        let style = Style::default().fg(color);
        Self { text, style }
    }

    fn as_span(&self) -> Span<'static> {
        Span::styled(self.text.clone(), self.style)
    }
}

#[derive(Debug, Clone)]
struct RichLine {
    texts: Vec<RichText>,
}

impl RichLine {
    fn new() -> Self {
        Self { texts: vec![] }
    }

    fn push(&mut self, text: RichText) {
        self.texts.push(text);
    }

    fn as_spans(&self) -> Spans<'static> {
        let mut spans = vec![];
        for text in &self.texts {
            spans.push(text.as_span());
        }
        Spans::from(spans)
    }

    /// Join two RichLines.
    fn join(&self, other: Self) -> Self {
        Self {texts: [&self.texts[..], &other.texts[..]].concat()}
    }

    fn extend(&mut self, texts: Vec<RichText>) {
        self.texts.extend(texts);
    }
}

struct App {
    pub lines: Vec<RichLine>,
    pub buffer: String,
    pub cursor_pos: usize,
    pub selection: Option<(usize, usize)>,
    pub tree: tree::TreeModel,
}

impl App {
    fn new(tree: tree::TreeModel) -> App {
        App {
            lines: Vec::new(),
            buffer: String::new(),
            cursor_pos: 0,
            selection: None,
            tree: tree,
        }
    }
    
    fn flush_buffer(&mut self) {
        let idx = self.lines.len() - 1;
        for p in self.rich_buffer() {
            self.lines[idx].push(p);
        }
        self.clear_buffer();
    }

    fn clear_buffer(&mut self) {
        self.buffer.clear();
        self.cursor_pos = 0;
    }
    
    fn print_prefix(&mut self, s: String, newline: bool) {
        if newline {
            let mut line = RichLine::new();
            line.push(RichText::new(s, Color::White));
            self.lines.push(line);
        }
        else {
            let idx = self.lines.len() - 1;
            self.lines[idx].push(RichText::new(s, Color::White));
        }
    }

    /// Get the vector of RichTexts from the buffer.
    fn rich_buffer(&self) -> Vec<RichText> {
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

    fn get_text(&self) -> Text {
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

fn process_keys<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> std::io::Result<String> {
    let _ = std::io::stdout().flush();  // flush stdout
    let prefix = app.tree.as_prefix();
    app.print_prefix(prefix, true);
    let output = loop {
        terminal.draw(|f| ui(f, &app))?;
        if let Event::Key(KeyEvent {code, modifiers, ..}) = event::read()? {
            match (code, modifiers) {
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    let output = app.buffer.clone();
                    app.buffer.push('\n');
                    app.flush_buffer();
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

pub fn enter(name: String) -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let root = get_json_path(&name)?;
    if !root.exists() {
        return Err(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Virtual directory {} does not exist.", name),
            )
        )?;
    }
    let tree = tree::TreeModel::from_file(&root)?;
    let mut app = App::new(tree);

    loop {
        // get valid input
        let user_input = process_keys(&mut terminal, &mut app)?;

        let input = match VCommand::from_string(&user_input){
            Ok(input) => input,
            Err(e) => {
                app.print_prefix(format!("{}", e), true);
                continue;
            }
        };
        let output = match input {
            VCommand::Empty => {
                Ok(())
            }
            VCommand::Cd { name } => {
                match name {
                    Some(path) => {
                        app.tree.move_by_string(&path)
                    }
                    None => {
                        app.tree.move_to_home()
                    }
                }
            
            }
            VCommand::Tree { name } => {
                match name {
                    Some(name) => {
                        let str = match app.tree.current.get_offspring(&name) {
                            Ok(item) => {
                                format!("{}", item)
                            }
                            Err(e) => {
                                format!("{}", e)
                            }
                        };
                        app.print_prefix(str, true);
                    }
                    None => {
                        app.print_prefix(format!("{}", app.tree.current), true);
                    }
                }
                Ok(())
            }
            VCommand::Ls { name, desc } => {
                let str = if desc {
                    app.tree.ls_with_desc(name)
                }
                else {
                    app.tree.ls_simple(name)
                };
                match str {
                    Ok(s) => {
                        app.print_prefix(format!("{}", s), true);
                        Ok(())
                    }
                    Err(e) => {
                        Err(e)
                    }
                }
            }
            VCommand::Pwd => {
                app.print_prefix(format!("./{}/{}", app.tree.root.name, app.tree.pwd()), true);
                Ok(())
            }
            VCommand::Cat { name } => {
                app.tree.print_file(&name)
            }
            VCommand::Touch { name } => {
                let vpath_cand = get_relative_vtree_path(true)?
                    .join(_VIRTUAL_FILES)
                    .join(name.clone());
                // find unique file name
                app.tree.create_new_file(&name, vpath_cand)
            }
            VCommand::Open { name } => {
                app.tree.open_file(&name)
            }
            VCommand::Cp { src, dst } => {
                app.tree.add_alias(dst, PathBuf::from(src))
            }
            VCommand::Desc { name, desc } => {
                let mut item = match name {
                    Some(name) => {
                        match app.tree.current.get_child_mut(&name){
                            Ok(item) => item,
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            },
                        }
                    }
                    None => {
                        &mut app.tree.current
                    }
                };
                match desc {
                    Some(desc) => {
                        // let mut item = item.clone();
                        item.desc = Some(desc);
                    }
                    None => {
                        // app.print_prefix(format!("{}", item.desc.as_ref().unwrap_or(&"".to_string())));
                    }
                }
                Ok(())
            }
            VCommand::Call { vec } => {
                app.tree.call_command(&vec)
            }
            VCommand::Mkdir { name } => {
                app.tree.mkdir(&name)
            }
            VCommand::Rm { name } => {
                match app.tree.current.get_child(&name) {
                    Ok(item) => {
                        match &item.entity {
                            Some(path) => {
                                std::fs::remove_file(path)?;
                            }
                            None => {}
                        }
                    }
                    Err(err) => {
                        app.print_prefix(format!("{}", err), true);
                        continue;
                    }
                };
                app.tree.rm(&name)
            }
            VCommand::Exit => {
                app.tree.to_file(root.as_path())?;
                break;
            }
        };
        match output {
            Ok(_) => {}
            Err(e) => {
                app.print_prefix(format!("{}", e), true);
            }
        }
    }
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
