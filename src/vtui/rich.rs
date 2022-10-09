use tui::{
    style::{Color, Style},
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

    /// Join two RichLines.
    pub fn join(&self, other: Self) -> Self {
        Self {texts: [&self.texts[..], &other.texts[..]].concat()}
    }

    pub fn extend(&mut self, texts: Vec<RichText>) {
        self.texts.extend(texts);
    }
}
