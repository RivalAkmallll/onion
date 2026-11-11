use std::fmt;
use std::ops::Range;

pub type Span = Range<usize>;

/// A common error type shared by every compiler stage.
#[derive(Debug, Clone)]
pub struct OnionError {
    pub message: String,
    pub span: Option<Span>,
    pub help: Option<String>,
}

impl OnionError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
            help: None,
        }
    }

    /// An error with no location in the source (e.g. "main function not found").
    pub fn without_span(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
            help: None,
        }
    }

    /// Attaches a suggestion shown as `= help: ...`.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Renders the error as readable text with line and column numbers.
    pub fn render(&self, source: &str, filename: &str) -> String {
        let Some(span) = &self.span else {
            let mut out = format!("error: {}\n  --> {}", self.message, filename);
            if let Some(help) = &self.help {
                out.push_str(&format!("\n  = help: {help}"));
            }
            return out;
        };

        // Make sure we never slice in the middle of a multi-byte character.
        let mut start = span.start.min(source.len());
        while !source.is_char_boundary(start) {
            start -= 1;
        }
        let mut end = span.end.min(source.len()).max(start);
        while !source.is_char_boundary(end) {
            end += 1;
        }

        let before = &source[..start];
        let line_no = before.matches('\n').count() + 1;
        let line_start = before.rfind('\n').map_or(0, |i| i + 1);
        let col = source[line_start..start].chars().count() + 1;
        let line_text = source[line_start..].lines().next().unwrap_or("");

        // Underline at most until the end of the line.
        let line_rest = line_text.chars().count().saturating_sub(col - 1).max(1);
        let width = source[start..end].chars().count().max(1).min(line_rest);

        let gutter = line_no.to_string();
        let pad = " ".repeat(gutter.len());

        let mut out = format!(
            "error: {msg}\n{pad} --> {file}:{line}:{col}\n{pad} |\n{gutter} | {text}\n{pad} | {spaces}{carets}",
            msg = self.message,
            file = filename,
            line = line_no,
            col = col,
            text = line_text,
            spaces = " ".repeat(col - 1),
            carets = "^".repeat(width),
        );

        if let Some(help) = &self.help {
            out.push_str(&format!("\n{pad} = help: {help}"));
        }
        out
    }
}

impl fmt::Display for OnionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for OnionError {}