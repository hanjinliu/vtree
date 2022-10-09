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

pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn new() -> Self {
        Self {start: 0, end: 0}
    }

    /// Canonicalize the selection, so that start <= end
    pub fn range(&self) -> (usize, usize) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }
    
    /// Get the size of the selection
    pub fn size(&self) -> usize {
        if self.start <= self.end {
            self.end - self.start
        } else {
            self.start - self.end
        }
    }
}
pub struct App {
    pub lines: History<RichLine>,
    pub buffer: String,
    pub cursor_pos: usize,
    pub selection: Selection,
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
            selection: Selection::new(),
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

    /// Equivalent to pushing BackSpace in terminal
    pub fn text_backspace_event(&mut self) {
        if self.selection.size() > 0 {
            let (start, end) = self.selection.range();
            self.buffer.replace_range(start..end, "");
            self.cursor_pos = start;
            self.selection = Selection::new();
            return;
        }
        if self.cursor_pos == 0 {
            return;
        }
        self.cursor_pos -= 1;
        self.buffer.remove(self.cursor_pos);
    }

    /// Equivalent to pushing Delete in terminal
    pub fn text_delete_event(&mut self) {
        if self.selection.size() > 0 {
            let (start, end) = self.selection.range();
            self.buffer.replace_range(start..end, "");
            self.cursor_pos = start;
            self.selection = Selection::new();
            return;
        }
        if self.cursor_pos == self.buffer.len() {
            return;
        }
        self.buffer.remove(self.cursor_pos);
    }

    pub fn text_move_cursor(&mut self, dx: i16, keep_selection: bool) {
        if 0 <= dx {
            let n = dx as usize;
            if self.cursor_pos + n <= self.buffer.len() {
                self.cursor_pos += n;
            }
        } 
        else {
            let n = -dx as usize;
            if self.cursor_pos >= n {
                self.cursor_pos -= n;
            }
        }
        if keep_selection {
            self.selection.end = self.cursor_pos;
        }
        else {
            self.selection.start = self.cursor_pos;
            self.selection.end = self.cursor_pos;
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