use std::ops::Range;

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet};
use pest::error::{ErrorVariant, InputLocation};

use crate::RaptorResult;
use raptor_parser::Rule;
use raptor_parser::dsl::Origin;

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

pub fn show_error_context(
    source: &str,
    source_path: impl AsRef<str>,
    title: &str,
    label: &str,
    err_range: Range<usize>,
) {
    let message = Level::ERROR.primary_title(title).element(
        Snippet::source(source)
            .fold(false)
            .annotation(AnnotationKind::Primary.span(err_range).label(label))
            .path(source_path.as_ref()),
    );

    let renderer = Renderer::styled();
    anstream::println!("{}", renderer.render(&[message]));
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

pub fn show_pest_error_context(raw: &str, err: &pest::error::Error<Rule>) -> RaptorResult<()> {
    let source_path = err.path().unwrap();

    let span = match err.location {
        InputLocation::Pos(idx) => index_to_line_remainder(raw, idx).unwrap_or(0..raw.len() - 1),
        InputLocation::Span((begin, end)) => begin..end,
    };

    let mut msg = err.variant.message();

    match &err.variant {
        ErrorVariant::ParsingError { positives, .. } if positives.len() == 1 => {
            match positives[0] {
                Rule::docker_source | Rule::from_source => {
                    msg = "Invalid FROM declaration. Specify the basename of a .rapt file, or a docker::<image>.".into();
                }

                _ => {}
            }
        }

        _ => {}
    }

    show_error_context(
        raw,
        source_path,
        &format!("parsing error: {msg}"),
        &msg,
        span,
    );
    Ok(())
}
