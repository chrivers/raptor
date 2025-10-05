use std::ops::Range;

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};

use crate::RaptorResult;
use raptor_parser::ast::Origin;
use raptor_parser::util::Location;

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

#[must_use]
pub fn context_lines(src: &str, range: Range<usize>, lines: usize) -> Range<usize> {
    let mut a = range.start;
    for _ in 0..lines {
        if let Some(m) = src[..a].rfind('\n') {
            a = m;
        } else {
            break;
        }
    }

    let mut b = range.end;
    for _ in 0..lines {
        if let Some(m) = src[b..].find('\n') {
            b += m + 1;
        } else {
            break;
        }
    }

    a..b
}

pub fn show_error_context(
    source: &str,
    source_path: impl AsRef<str>,
    title: &str,
    label: &str,
    err_range: Range<usize>,
) {
    let visible_range = context_lines(source, err_range.clone(), 3);

    let message = Level::ERROR.primary_title(title).element(
        Snippet::source(source)
            .fold(true)
            .annotation(AnnotationKind::Primary.span(err_range).label(label))
            .annotation(AnnotationKind::Visible.span(visible_range))
            .path(source_path.as_ref()),
    );

    let renderer = Renderer::styled();
    anstream::eprintln!("{}", renderer.render(&[message]));
}

pub fn show_origin_error_context(source: &str, origin: &Origin, title: &str, label: &str) {
    show_error_context(
        source,
        origin.path.as_ref(),
        title,
        label,
        origin.span.clone(),
    );
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

    show_error_context(&raw, source_path, title, &label, err_range);
    Ok(())
}

pub fn show_parse_error_context(
    source: &str,
    err: &Location<raptor_parser::ParseError>,
) -> RaptorResult<()> {
    let source_path = err.origin.path.as_str();
    let title = "Parse error";
    let label = err.to_string();
    let err_range = err.origin.span.clone();

    show_error_context(source, source_path, title, &label, err_range);

    Ok(())
}
