use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::Widget;
use tui_big_text::{BigText, PixelSize};

pub struct RaptorLogo<'a> {
    text: BigText<'a>,
}

impl RaptorLogo<'_> {
    #[must_use]
    pub fn complete() -> Self {
        let text = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .lines(vec![
                "Raptor complete".light_blue().bold().into(),
                "---------------".dark_gray().into(),
                "press q to quit".white().into(),
            ])
            .centered()
            .build();

        Self { text }
    }

    #[must_use]
    pub fn failed() -> Self {
        let text = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .lines(vec![
                "<Build failure>".light_red().bold().into(),
                "---------------".dark_gray().into(),
                "press q to quit".white().into(),
            ])
            .centered()
            .build();

        Self { text }
    }
}

impl Widget for RaptorLogo<'_> {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rect = center(
            area,
            Constraint::Fill(1),
            Constraint::Length(self.text.lines.len() as u16 * 4),
        );

        self.text.render(rect, buf);
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
