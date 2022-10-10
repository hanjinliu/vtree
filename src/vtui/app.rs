use tui::{
    style::{Color, Style},
    text::Text,
};
use super::{
    rich::{RichText, RichLine},
    history::History,
    super::{
        terminal::parse_string_raw,
        tree,
    },
};

const _VIRTUAL_FILES: &str = "virtual-files";

pub struct Cursor {
    start: usize,
    pos: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self {start: 0, pos: 0}
    }

    /// Canonicalize the selection, so that start <= end
    pub fn selection(&self) -> (usize, usize) {
        if self.start <= self.pos {
            (self.start, self.pos)
        } else {
            (self.pos, self.start)
        }
    }
    
    /// Get the size of the selection
    pub fn selection_size(&self) -> usize {
        if self.start <= self.pos {
            self.pos - self.start
        } else {
            self.start - self.pos
        }
    }

    pub fn move_to(&mut self, pos: usize) {
        self.start = pos;
        self.pos = self.start;
    }

    pub fn select_to(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn clear_selection(&mut self) {
        self.start = self.pos;
    }

    pub fn set_selection(&mut self, start: usize, end: usize) {
        self.start = start;
        self.pos = end;
    }

}

pub struct TabCompleter {
    candidates: History<String>,
    seed: String,
}

impl TabCompleter {
    pub fn new() -> Self {
        Self {
            candidates: History::new(100),
            seed: String::new(),
        }
    }

    pub fn set_seed(&mut self, seed: &String) {
        if &self.seed != seed {
            self.candidates.index = 0;
            self.seed = seed.clone();
        }
    }

    pub fn candidates_from(&mut self, list: Vec<String>) {
        self.candidates.clear();
        for item in list {
            if item.starts_with(&self.seed) {
                self.candidates.add(item);
            }
        }
        self.candidates.index = 0;
    }

    pub fn next(&mut self) -> Option<String> {
        self.candidates.next()
    }

    pub fn prev(&mut self) -> Option<String> {
        self.candidates.prev()
    }

    pub fn candidates(&self) -> Vec<String> {
        let mut vec = Vec::new();
        for h in self.candidates.history.iter() {
            vec.push(h.clone());
        }
        vec
    }
}

pub struct App {
    pub lines: History<RichLine>,
    pub buffer: String,
    pub cursor: Cursor,
    pub tree: tree::TreeModel,
    pub history: History<String>,
    pub tab_completion: TabCompleter,
    pub scroll_pos: usize,
}

impl App {
    pub fn new(tree: tree::TreeModel) -> Self {
        Self {
            lines: History::new(1000),
            buffer: String::new(),
            cursor: Cursor::new(),
            tree,
            history: History::new(500),
            tab_completion: TabCompleter::new(),
            scroll_pos: 0,
        }
    }
    
    /// Clear current buffer string and add it to history
    pub fn flush_buffer(&mut self) {
        let idx = self.lines.len() - 1;
        let buf = self.rich_buffer();
        self.lines[idx].extend(buf);
        self.clear_buffer();
    }

    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
        self.cursor.move_to(0);
    }

    pub fn run_buffer(&mut self) {
        let buf = self.buffer.clone();
        self.history.add(buf);
        self.flush_buffer();
    }

    pub fn set_buffer(&mut self, buf: String) {
        self.buffer = buf;
        self.cursor.move_to(self.buffer.len());
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
    pub fn rich_buffer(&self) -> RichLine {
        let strs = parse_string_raw(&self.buffer);
        let nstr = strs.len();
        if nstr == 0 {
            // Show a single space (corresponding to the cursor) if the buffer is empty.
            let text = RichText::new(" ".to_string(), Color::Black)
                .restyled(Style::default().bg(Color::Rgb(108, 108, 108)));
            let mut line = RichLine::new();
            line.push(text);
            return line;
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
                    args.push(RichText::new(str.to_string(), Color::Blue));
                } else {
                    args.push(RichText::new(str.to_string(), Color::White));
                }
            }

            let mut line = RichLine::from(args);
            // style selected text
            if self.cursor.selection_size() > 0 {
                let (start, end) = self.cursor.selection();
                line = line.restyled(
                    start, end,
                    Style::default().fg(Color::Black).bg(Color::Rgb(128, 128, 128))
                );
            }
            // style cursor
            if self.cursor.pos < self.buffer.len() {
                line = line.restyled(
                    self.cursor.pos, self.cursor.pos + 1,
                    Style::default().fg(Color::Black).bg(Color::Rgb(108, 108, 108))
                );
            } else {
                line.push(RichText::new(" ".to_string(), Color::Black));
                line = line.restyled(
                    self.cursor.pos, self.cursor.pos + 1,
                    Style::default().fg(Color::Black).bg(Color::Rgb(108, 108, 108))
                );
            }
            return line;
        }
    }

    /// Equivalent to pushing BackSpace in terminal
    pub fn text_backspace_event(&mut self) {
        if self.cursor.selection_size() > 0 {
            self.clear_selected_text();
            return;
        }
        if self.cursor.pos == 0 {
            return;
        }
        self.cursor.move_to(self.cursor.pos - 1);
        self.buffer.remove(self.cursor.pos);
    }

    /// Equivalent to pushing Delete in terminal
    pub fn text_delete_event(&mut self) {
        if self.cursor.selection_size() > 0 {
            self.clear_selected_text();
            return;
        }
        if self.cursor.pos == self.buffer.len() {
            return;
        }
        self.buffer.remove(self.cursor.pos);
    }

    pub fn text_move_cursor(&mut self, dx: i16, keep_selection: bool) {
        if 0 <= dx {
            let n = dx as usize;
            if self.cursor.pos + n <= self.buffer.len() {
                if keep_selection {
                    self.cursor.select_to(self.cursor.pos + n);
                } else {
                    self.cursor.move_to(self.cursor.pos + n);
                }
            } else {
                self.cursor.clear_selection();
            }
        } else {
            let n = -dx as usize;
            if self.cursor.pos >= n {
                if keep_selection {
                    self.cursor.select_to(self.cursor.pos - n);
                } else {
                    self.cursor.move_to(self.cursor.pos - n);
                }
            } else {
                self.cursor.clear_selection();
            }
        }
    }

    pub fn text_move_cursor_to_next_word(&mut self, keep_selection: bool) {
        let mut pos = self.cursor.pos;
        let mut whitespace_found = false;
        
        self.buffer[self.cursor.pos..]
            .chars()
            .enumerate()
            .find(|(i, c)| {
                if !c.is_whitespace() {
                    pos = self.cursor.pos + i;
                    whitespace_found
                } else {
                    whitespace_found = true;
                    false
                }
            })  
            .or_else(|| {
                pos = self.buffer.len();
                None
            });

        if keep_selection {
            self.cursor.select_to(pos);
        } else {
            self.cursor.move_to(pos);
        }
    }

    pub fn text_move_cursor_to_prev_word(&mut self, keep_selection: bool) {
        let mut pos = self.cursor.pos;
        let mut char_found = false;
        
        self.buffer[..self.cursor.pos]
            .chars()
            .rev()
            .enumerate()
            .find(|(i, c)| {
                if c.is_whitespace() {
                    pos = self.cursor.pos - i;
                    char_found
                } else {
                    char_found = true;
                    false
                }
            })  
            .or_else(|| {
                pos = 0;
                None
            });

        if keep_selection {
            self.cursor.select_to(pos);
        } else {
            self.cursor.move_to(pos);
        }
    }

    pub fn text_add_char(&mut self, c: char) {
        if self.cursor.selection_size() > 0 {
            self.clear_selected_text();
        }
        self.buffer.insert(self.cursor.pos, c);
        self.cursor.move_to(self.cursor.pos + 1);
    }

    // Get the selected text.
    pub fn text_selected(&mut self) -> String {
        let (start, end) = self.cursor.selection();
        self.buffer[start..end].to_string()
    }

    /// Insert text at the cursor position
    pub fn insert_text(&mut self, text: String) {
        self.buffer.insert_str(self.cursor.pos, &text);
        self.cursor.move_to(self.cursor.pos + text.len());
    }

    fn clear_selected_text(&mut self) {
        let (start, end) = self.cursor.selection();
        self.buffer.replace_range(start..end, "");
        self.cursor.move_to(start);
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