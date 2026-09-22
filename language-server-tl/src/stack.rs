//! Stack-state inspection via hover for ToastLang.
//!
//! In a concatenative language the constant question is "what is on the stack
//! right now?". The type checker knows, via `tlc::analyze_mapped`, which records
//! the stack after each statement and at each `??` hole. Hovering a word shows
//! the stack at that point.

use tower_lsp::lsp_types::*;

/// Hover: show the type stack at `position`.
pub fn hover(doc_uri: &Url, doc_text: &str, position: Position) -> Option<Hover> {
    // Include resolution (and thus stack info across includes) needs a real path.
    let path = doc_uri.to_file_path().ok()?;
    let doc_path = path.to_string_lossy().into_owned();
    let include_paths = crate::include_paths_for(&path);

    let analysis = tlc::analyze_mapped(doc_text, &doc_path, &include_paths);
    let offset = crate::position_to_byte(doc_text, position);

    // Among probes in this document whose span covers the cursor, prefer a `??`
    // hole, then the innermost (latest-starting) statement.
    let chosen = analysis
        .probes
        .iter()
        .filter(|p| p.file == doc_path && p.start <= offset && offset <= p.end)
        .max_by_key(|p| (p.is_hole, p.start))?;

    let stack = if chosen.stack.is_empty() {
        "*empty*".to_string()
    } else {
        format!("`{}`", chosen.stack)
    };
    let label = if chosen.is_hole { "Stack at `??`" } else { "Stack" };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("{label}: {stack}"),
        }),
        range: Some(Range {
            start: crate::byte_to_position(doc_text, chosen.start),
            end: crate::byte_to_position(doc_text, chosen.end),
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hover_value(uri: &str, src: &str, line: u32, ch: u32) -> Option<String> {
        let uri = Url::parse(uri).unwrap();
        match hover(&uri, src, Position { line, character: ch })?.contents {
            HoverContents::Markup(m) => Some(m.value),
            _ => None,
        }
    }

    #[test]
    fn hover_shows_stack_on_statement() {
        // `fn main -> { 1 2 + }` — cursor on `2` (col 15): stack is `u16 u16`.
        let v = hover_value("file:///tmp/x.tl", "fn main -> { 1 2 + }", 0, 15).unwrap();
        assert!(v.contains("u16 u16"), "got: {v}");
    }

    #[test]
    fn hover_shows_hole_stack() {
        // `??` after `1 2`: hole stack is `u16 u16`, labelled as a hole.
        let v = hover_value("file:///tmp/x.tl", "fn main -> { 1 2 ?? }", 0, 17).unwrap();
        assert!(v.contains("??"), "expected hole label, got: {v}");
        assert!(v.contains("u16 u16"), "got: {v}");
    }

    #[test]
    fn hover_on_whitespace_is_none() {
        // Column 10 is whitespace between `->` and `{` — no probe covers it.
        assert!(hover_value("file:///tmp/x.tl", "fn main -> { 1 }", 0, 10).is_none());
    }
}
