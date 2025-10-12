use ratatui::style::{Color, Style};

#[derive(Debug, Clone, Copy)]
pub enum JobState {
    Planned,
    Running,
    Completed,
}

impl JobState {
    #[must_use]
    pub const fn symbol(self, index: usize) -> &'static str {
        const P1: &[&str] = &["⬖", "⬗", "⬘", "⬙"];
        /* const P2: &[&str] = &["⤴", "⤵", "⤶", "⤷"]; */
        match self {
            Self::Planned => "✚",
            Self::Running => P1[index % 4],
            Self::Completed => "🗹",
        }
    }

    #[must_use]
    pub const fn color(self) -> Style {
        match self {
            Self::Planned => Style::new().fg(Color::White),
            Self::Running => Style::new().fg(Color::LightBlue),
            Self::Completed => Style::new().fg(Color::LightGreen),
        }
    }
}
