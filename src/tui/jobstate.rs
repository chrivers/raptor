use ratatui::style::{Color, Style};
use throbber_widgets_tui::QUADRANT_BLOCK_CRACK;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Planned,
    Running,
    Completed,
    Failed,
}

impl JobState {
    #[must_use]
    pub const fn symbol(self, index: usize) -> &'static str {
        let table = QUADRANT_BLOCK_CRACK;
        match self {
            Self::Planned => "✚",
            Self::Running => table.symbols[index % table.symbols.len()],
            Self::Completed => "🗹",
            Self::Failed => "X",
        }
    }

    #[must_use]
    pub const fn color(self) -> Style {
        match self {
            Self::Planned => Style::new().fg(Color::White),
            Self::Running => Style::new().fg(Color::LightBlue),
            Self::Completed => Style::new().fg(Color::LightGreen),
            Self::Failed => Style::new().fg(Color::LightRed),
        }
    }
}
