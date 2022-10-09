use tui::{
    style::Color,
    text::Text,
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

}
pub struct App {
    pub lines: History<RichLine>,
    pub buffer: String,
    pub cursor: Cursor,
    pub tree: tree::TreeModel,
    pub history: History<String>,
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
            }
        } 
        else {
            let n = -dx as usize;
            if self.cursor.pos >= n {
                if keep_selection {
                    self.cursor.select_to(self.cursor.pos - n);
                } else {
                    self.cursor.move_to(self.cursor.pos - n);
                }
            }
        }
    }

    pub fn text_add_char(&mut self, c: char) {
        if self.cursor.selection_size() > 0 {
            self.clear_selected_text();
        }
        self.buffer.insert(self.cursor.pos, c);
        self.cursor.move_to(self.cursor.pos + 1);
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