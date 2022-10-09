use tui::{
    style::{Color, Style, Modifier},
    text::{Span, Spans},
};

#[derive(Debug, Clone)]
pub struct RichText {
    text: String,
    style: Style,
}

impl RichText {
    pub fn new(text: String, color: Color) -> Self {
        let style = Style::default().fg(color);
        Self { text, style }
    }

    pub fn as_span(&self) -> Span<'static> {
        Span::styled(self.text.clone(), self.style)
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Get subset of the text with same style
    pub fn subtext(&self, start: usize, end: usize) -> Self {
        let text = self.text[start..end].to_string();
        Self { text, style: self.style }
    }

    pub fn restyled(&self, style: Style) -> Self {
        let new_style = match style.fg {
            Some(fg) => self.style.fg(fg),
            None => self.style,
        };
        let new_style = match style.bg {
            Some(bg) => new_style.bg(bg),
            None => new_style,
        };
        let new_style = if style.add_modifier == Modifier::empty() {
            new_style
        } else {
            new_style.add_modifier(style.add_modifier)
        };
        Self { text: self.text.clone(), style: new_style }
    }

    fn split_at(&self, pos: usize) -> (Self, Self) {
        let (left, right) = self.text.split_at(pos);
        let left = Self { text: left.to_string(), style: self.style };
        let right = Self { text: right.to_string(), style: self.style };
        (left, right)
    }

    fn split3_at(&self, pos0: usize, pos1: usize) -> (Self, Self, Self) {
        let (left, mid_right) = self.text.split_at(pos0);
        let (middle, right) = mid_right.split_at(pos1 - pos0);
        let left = Self { text: left.to_string(), style: self.style };
        let middle = Self { text: middle.to_string(), style: self.style };
        let right = Self { text: right.to_string(), style: self.style };
        (left, middle, right)
    }
}


#[derive(Debug, Clone)]
pub struct RichLine {
    texts: Vec<RichText>,
}

impl RichLine {
    pub fn new() -> Self {
        Self { texts: vec![] }
    }

    pub fn push(&mut self, text: RichText) {
        self.texts.push(text);
    }

    pub fn as_spans(&self) -> Spans<'static> {
        let mut spans = vec![];
        for text in &self.texts {
            spans.push(text.as_span());
        }
        Spans::from(spans)
    }

    pub fn iter_texts(&self) -> impl Iterator<Item = &RichText> {
        self.texts.iter()
    }

    /// Return the unstyled String.
    pub fn raw_text(&self) -> String {
        let mut text = String::new();
        for t in &self.texts {
            text.push_str(&t.text);
        }
        text
    }

    /// Join two RichLines.
    pub fn join(&self, other: Self) -> Self {
        Self {texts: [&self.texts[..], &other.texts[..]].concat()}
    }

    /// Extend a RichLine with a vector of RichTexts.
    pub fn extend(&mut self, line: Self) {
        self.texts.extend(line.texts);
    }

    /// Partially change the style of the texts.
    pub fn restyled(&self, start: usize, end: usize, style: Style) -> Self {
        let mut pos = 0;
        let mut sl = -1..-1;
        let mut start_residue = 0;
        let mut end_residue = 0;
        
        // Find the start and end indices
        for (idx, rtext) in self.texts.iter().enumerate() {
            let next_pos = pos + rtext.len();
            if next_pos <= start {
                pos = next_pos;
                continue;
            }
            if sl.start < 0 {
                start_residue = start - pos;
                sl.start = idx as i32;
            }
            
            if next_pos >= end {
                end_residue = end - pos;
                sl.end = idx as i32 + 1;
                break;
            }
            pos = next_pos;
        }
        
        // Update the style
        let sl = sl.start as usize..sl.end as usize;
        let mut updated = Vec::new();
        let new_texts = 
            if sl.start == sl.end - 1 {
                let (rt0, rt1, rt2) = self
                    .texts[sl.start].
                    split3_at(start_residue, end_residue);
                updated.extend([rt0, rt1.restyled(style), rt2]);
                vec![&self.texts[..sl.start], &updated, &self.texts[sl.end..]]
            } else {
                let niter = sl.len();
                let mut iter = self.texts[sl.clone()].iter();

                // First text
                let (rt0, rt1) = iter.next().unwrap().split_at(start_residue);
                updated.push(rt0);
                updated.push(rt1.restyled(style));

                // Middle texts
                for _ in 1..niter - 1 {
                    let rt = iter.next().unwrap();
                    updated.push(rt.restyled(style));
                }

                // Last text
                let (rt1, rt2) = iter.next().unwrap().split_at(end_residue);
                updated.push(rt1.restyled(style));
                updated.push(rt2);
                vec![&self.texts[..sl.start], &updated[..], &self.texts[sl.end..]]
            };
        
        Self { texts: new_texts.concat() }
    }
}

impl From<Vec<RichText>> for RichLine {
    fn from(texts: Vec<RichText>) -> Self {
        Self { texts }
    }
}

impl From<Vec<String>> for RichLine {
    fn from(texts: Vec<String>) -> Self {
        let texts = texts
            .into_iter()
            .map(|text| RichText::new(text, Color::White))
            .collect();
        Self { texts }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rich_text() {
        let text = RichText::new("Hello World".to_string(), Color::White);
        assert_eq!(text.len(), 11);
        assert_eq!(text.text, "Hello World");
        assert_eq!(text.style.fg, Some(Color::White));
        assert_eq!(text.style.bg, None);
        assert_eq!(text.style.add_modifier, Modifier::empty());
        assert_eq!(text.style.sub_modifier, Modifier::empty());
    }

    #[test]
    fn test_rich_text_split_at() {
        let text = RichText::new("Hello World".to_string(), Color::White);
        let (left, right) = text.split_at(5);
        assert_eq!(left.text, "Hello");
        assert_eq!(right.text, " World");
    }

    #[test]
    fn test_rich_text_split3_at() {
        let text = RichText::new("Hello World".to_string(), Color::White);
        let (left, middle, right) = text.split3_at(5, 7);
        assert_eq!(left.text, "Hello");
        assert_eq!(middle.text, " W");
        assert_eq!(right.text, "orld");
    }

    #[test]
    fn test_rich_text_restyled() {
        let text = RichText::new("Hello World".to_string(), Color::White);
        let text = text.restyled(Style::default().fg(Color::Red));
        assert_eq!(text.text, "Hello World");
        assert_eq!(text.style.fg, Some(Color::Red));
        assert_eq!(text.style.bg, None);
        assert_eq!(text.style.add_modifier, Modifier::empty());
        assert_eq!(text.style.sub_modifier, Modifier::empty());
    }

    #[test]
    fn test_rich_line() {
        let line = RichLine::new();
        assert_eq!(line.texts.len(), 0);
    }

    #[test]
    fn test_rich_line_push() {
        let mut line = RichLine::new();
        line.push(RichText::new("Hello World".to_string(), Color::White));
        assert_eq!(line.texts.len(), 1);
        assert_eq!(line.texts[0].text, "Hello World");
        assert_eq!(line.texts[0].style.fg, Some(Color::White));
    }

    #[test]
    fn test_rich_line_restyled_raw_text() {
        let line = RichLine::from(vec!["Hello ".to_string(), "World".to_string()]);
        assert_eq!(line.raw_text(), "Hello World");
        assert_eq!(line.restyled(1, 3, Style::default().fg(Color::Red)).raw_text(), "Hello World");
        assert_eq!(line.restyled(1, 8, Style::default().fg(Color::Red)).raw_text(), "Hello World");
        assert_eq!(line.restyled(7, 8, Style::default().fg(Color::Red)).raw_text(), "Hello World");
    }
}
