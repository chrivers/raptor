use std::ops::Range;

use annotate_snippets::{AnnotationKind, Element, Level, Renderer, Snippet};

use crate::RaptorResult;
use raptor_parser::{ParseError, ParseErrorDetails, ast::Origin};

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

pub fn show_error_context<'a>(
    source: &str,
    source_path: impl AsRef<str>,
    title: &str,
    label: &str,
    err_range: Range<usize>,
    elements: impl IntoIterator<Item = Element<'a>>,
) {
    let mut message = Level::ERROR.primary_title(title).element(
        Snippet::source(source)
            .fold(false)
            .annotation(AnnotationKind::Primary.span(err_range).label(label))
            .path(source_path.as_ref()),
    );

    for element in elements {
        message = message.element(element);
    }

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
        [],
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

    show_error_context(&raw, source_path, title, &label, err_range, []);
    Ok(())
}

pub fn show_pest_error_context(raw: &str, err: &ParseError) -> RaptorResult<()> {
    let source_path = err.path.as_str();

    let span = match err.details {
        ParseErrorDetails::PathParse(_) | ParseErrorDetails::ParseError(_) => {
            index_to_line_remainder(raw, 0).unwrap_or(0..raw.len() - 1)
        }
        ParseErrorDetails::InvalidToken(pos) | ParseErrorDetails::UnexpectedEof(pos, _) => {
            index_to_line_remainder(raw, pos).unwrap_or(0..raw.len() - 1)
        }
        ParseErrorDetails::UnrecognizedToken {
            token: (start, _, end),
            ..
        }
        | ParseErrorDetails::UnexpectedToken {
            token: (start, _, end),
        } => start..end,
    };

    let mut elements = vec![];

    let make_expected = |exp: &[String]| {
        Level::NOTE
            .with_name("Expected one of")
            .message(exp.join(", "))
            .into()
    };

    let msg = match &err.details {
        ParseErrorDetails::PathParse(_) => "Path parse error".into(),
        ParseErrorDetails::InvalidToken(_) => "Invalid token".into(),
        ParseErrorDetails::UnexpectedEof(_, expected) => {
            elements.push(make_expected(expected));
            "Unexpected end of file".into()
        }
        ParseErrorDetails::UnrecognizedToken { expected, .. } => {
            elements.push(make_expected(expected));
            "Unrecognized token".into()
        }
        ParseErrorDetails::UnexpectedToken { token: (_, tok, _) } => {
            format!("Unexpected token {tok}")
        }
        ParseErrorDetails::ParseError(err) => format!("parse error: {err}"),
    };

    show_error_context(
        raw,
        source_path,
        &format!("parsing error: {msg}"),
        &msg,
        span,
        elements,
    );

    Ok(())
}
