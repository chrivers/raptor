use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

#[derive(Default)]
pub struct StatusBar<'a> {
    spans: Vec<Span<'a>>,
}

impl<'a> StatusBar<'a> {
    #[must_use]
    pub const fn new() -> Self {
        Self { spans: Vec::new() }
    }

    pub fn add<S: Into<Span<'a>>>(&mut self, span: S) {
        self.spans.push(span.into());
        self.separator();
    }

    pub fn counter<S: Into<Span<'a>>>(&mut self, number: usize, span: S) {
        self.spans
            .push(Span::raw(format!("{number:5} ")).bold().white());
        self.spans.push(span.into());

        self.separator();
    }

    pub fn separator(&mut self) {
        self.spans.push(Span::raw(" | ").white().bold());
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let line = Line::from(self.spans).style(Style::new().bg(Color::Blue));

        line.render(area, buf);
    }
}
