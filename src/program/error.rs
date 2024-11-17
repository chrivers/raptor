use std::ops::Range;

use annotate_snippets::{Level, Renderer, Snippet};

use crate::RaptorResult;

#[must_use]
pub fn line_number_to_span(text: &str, line: usize) -> Range<usize> {
    let (a, b) = text
        .lines()
        .take(line)
        .fold((0, 0), |(_, b), l| (b, b + l.len() + 1));

    a..b
}

pub fn show_error_context(
    source_path: &str,
    title: &str,
    label: &str,
    err_range: Range<usize>,
) -> RaptorResult<()> {
    let raw = std::fs::read_to_string(source_path)?;

    let message = Level::Error.title(title).snippet(
        Snippet::source(&raw)
            .fold(false)
            .origin(source_path)
            .annotation(Level::Error.span(err_range).label(label)),
    );

    let renderer = Renderer::styled();
    anstream::println!("{}", renderer.render(message));

    Ok(())
}

pub fn show_jinja_error_context(err: &minijinja::Error) -> RaptorResult<()> {
    let source_path = err.name().unwrap();
    let raw = std::fs::read_to_string(source_path)?;

    let title = err.detail().unwrap_or("<unknown error>");
    let label = format!("{err}");

    let err_range = err
        .range()
        .or_else(|| err.line().map(|line| line_number_to_span(&raw, line)))
        .unwrap_or(0..raw.len() - 1);

    let message = Level::Error.title(title).snippet(
        Snippet::source(&raw)
            .fold(false)
            .origin(source_path)
            .annotation(Level::Error.span(err_range).label(&label)),
    );

    let renderer = Renderer::styled();
    anstream::println!("{}", renderer.render(message));

    Ok(())
}
