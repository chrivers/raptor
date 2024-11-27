use anstyle::{AnsiColor, Color, Style};

#[must_use]
pub const fn style() -> clap::builder::Styles {
    let style = Style::new();
    let fg_cyan = style.fg_color(Some(Color::Ansi(AnsiColor::Cyan)));
    let fg_green = style.fg_color(Some(Color::Ansi(AnsiColor::Green)));
    let fg_red = style.fg_color(Some(Color::Ansi(AnsiColor::Red)));
    let fg_white = style.fg_color(Some(Color::Ansi(AnsiColor::White)));
    let fg_yellow = style.fg_color(Some(Color::Ansi(AnsiColor::Yellow)));

    clap::builder::Styles::styled()
        .usage(fg_white.bold())
        .header(fg_yellow.bold())
        .literal(fg_green)
        .invalid(fg_red.bold())
        .error(fg_red.bold())
        .valid(fg_green.bold())
        .placeholder(fg_cyan)
}
