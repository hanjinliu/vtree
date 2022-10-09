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

    /// Join two RichLines.
    pub fn join(&self, other: Self) -> Self {
        Self {texts: [&self.texts[..], &other.texts[..]].concat()}
    }

    /// Extend a RichLine with a vector of RichTexts.
    pub fn extend(&mut self, line: Self) {
        self.texts.extend(line.texts);
    }

    /// Partially change the style.
    pub fn restyled(&self, start: usize, end: usize, style: Style) -> Self {
        let mut pos = 0;
        let mut idx_start = -1;
        let mut idx_end = -1;
        let mut start_residue = 0;
        let mut end_residue = 0;
        
        // Find the start and end indices
        for (idx, rtext) in self.texts.iter().enumerate() {
            let next_pos = pos + rtext.len();
            if next_pos <= start {
                continue;
            }
            if idx_start < 0 {
                start_residue = start - pos;
                idx_start = idx as i32;
            }
            
            if next_pos >= end {
                end_residue = end - pos;
                idx_end = idx as i32 + 1;
                break;
            }
            pos = next_pos;
        }
        
        // Update the style
        let idx_start = idx_start as usize;
        let idx_end = idx_end as usize;
        let mut updated = Vec::new();
        let new_texts = 
            if idx_start == idx_end - 1 {
                let (rt0, rt1, rt2) = self
                    .texts[idx_start].
                    split3_at(start_residue, end_residue);
                updated.extend([rt0, rt1.restyled(style), rt2]);
                vec![&self.texts[..idx_start], &updated, &self.texts[idx_end..]]
            } else {
                let niter = idx_end - idx_start;
                let mut iter = self.texts[idx_start..idx_end].iter();

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
                vec![&self.texts[..idx_start], &updated[..], &self.texts[idx_end..]]
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