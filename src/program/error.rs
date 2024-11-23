use std::ops::Range;

use annotate_snippets::{Level, Renderer, Snippet};

use crate::{parser::Rule, RaptorResult};

#[must_use]
pub fn line_number_to_span(text: &str, line: usize) -> Range<usize> {
    let (a, b) = text
        .lines()
        .take(line)
        .fold((0, 0), |(_, b), l| (b, b + l.len() + 1));

    a..b
}

#[must_use]
pub fn index_to_line_remainder(text: &str, idx: usize) -> Option<Range<usize>> {
    let mut size = 0;
    for ln in text.lines() {
        let line_end = size + ln.len();
        if line_end > idx {
            return Some(idx..line_end);
        }
        size = line_end + 1;
    }

    if size >= idx {
        return Some(idx..size);
    }

    None
}

fn _show_error_context(
    raw: &str,
    source_path: &str,
    title: &str,
    label: &str,
    err_range: Range<usize>,
) {
    let message = Level::Error.title(title).snippet(
        Snippet::source(raw)
            .fold(false)
            .origin(source_path)
            .annotation(Level::Error.span(err_range).label(label)),
    );

    let renderer = Renderer::styled();
    anstream::println!("{}", renderer.render(message));
}

pub fn show_error_context(
    source_path: &str,
    title: &str,
    label: &str,
    err_range: Range<usize>,
) -> RaptorResult<()> {
    let raw = std::fs::read_to_string(source_path)?;

    _show_error_context(&raw, source_path, title, label, err_range);
    Ok(())
}

pub fn show_jinja_error_context(err: &minijinja::Error) -> RaptorResult<()> {
    let source_path = err.name().unwrap();
    let raw = std::fs::read_to_string(source_path)?;

    let kind_desc = err.kind().to_string();
    let title = err.detail().unwrap_or(&kind_desc);
    let label = format!("{err}");

    let err_range = err
        .range()
        .or_else(|| err.line().map(|line| line_number_to_span(&raw, line)))
        .unwrap_or(0..raw.len() - 1);

    _show_error_context(&raw, source_path, title, &label, err_range);
    Ok(())
}

pub fn show_pest_error_context(err: &pest::error::Error<Rule>) -> RaptorResult<()> {
    let source_path = err.path().unwrap();
    let raw = std::fs::read_to_string(source_path)?;

    let span = match err.location {
        pest::error::InputLocation::Pos(idx) => {
            index_to_line_remainder(&raw, idx).unwrap_or(0..raw.len() - 1)
        }
        pest::error::InputLocation::Span((begin, end)) => begin..end,
    };

    _show_error_context(
        &raw,
        source_path,
        &format!("{}", err.variant),
        &err.variant.message(),
        span,
    );
    Ok(())
}
