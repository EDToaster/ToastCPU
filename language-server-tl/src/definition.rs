//! Go-to-definition for ToastLang.
//!
//! Builds a cross-file symbol index over the document and everything it
//! `#include`s (reusing `tlc::included_files`), then resolves the identifier
//! under the cursor to the definition(s) of a top-level item — function,
//! struct, global, inline, or module.
//!
//! v1 scope: top-level definitions only. Local `let` bindings, function
//! parameters, and struct-field (`.field`) access are not yet resolved.

use std::fs;

use tlc::parser_util::types::Module;
use tlc::{Span, TokenKind};
use tower_lsp::lsp_types::{Location, Position, Range, Url};

/// A definition site collected from the include graph.
struct Symbol {
    /// Fully module-qualified name, e.g. `io::print`.
    qualified: String,
    /// Bare name, e.g. `print`.
    simple: String,
    location: Location,
}

/// Resolve the identifier at `position` in `doc_text` to its definition(s).
pub fn goto(doc_uri: &Url, doc_text: &str, position: Position) -> Vec<Location> {
    let offset = position_to_byte(doc_text, position);
    let word = match identifier_at(doc_text, offset) {
        Some(w) => w,
        None => return Vec::new(),
    };
    let index = build_index(doc_uri, doc_text);
    resolve(&word, &index)
}

/// The identifier token covering byte `offset`, if any.
fn identifier_at(text: &str, offset: usize) -> Option<String> {
    for t in tlc::lex(text) {
        if t.kind == TokenKind::Ident && t.start <= offset && offset <= t.end {
            return Some(text[t.start..t.end].to_string());
        }
    }
    None
}

/// Build the symbol index over the document and its include graph.
fn build_index(doc_uri: &Url, doc_text: &str) -> Vec<Symbol> {
    let mut out = Vec::new();

    // Non-file document (e.g. an unsaved buffer): index just the document.
    let doc_path = match doc_uri.to_file_path() {
        Ok(p) => p,
        Err(_) => {
            if let (Some(module), _) = tlc::parse(doc_text) {
                collect(&module, "", doc_uri, doc_text, &mut out);
            }
            return out;
        }
    };
    let doc_path_str = doc_path.to_string_lossy().into_owned();
    let include_paths = crate::include_paths_for(&doc_path);

    for file in tlc::included_files(doc_text, &doc_path_str, &include_paths) {
        // Use the live buffer for the open document; read the rest from disk.
        let (content, uri) = if file == doc_path_str {
            (doc_text.to_string(), doc_uri.clone())
        } else {
            match (fs::read_to_string(&file), Url::from_file_path(&file)) {
                (Ok(c), Ok(u)) => (c, u),
                _ => continue,
            }
        };
        if let (Some(module), _) = tlc::parse(&content) {
            collect(&module, "", &uri, &content, &mut out);
        }
    }
    out
}

/// Walk a module tree, recording each top-level definition with its
/// module-qualified name and the location of its name.
fn collect(module: &Module, prefix: &str, uri: &Url, content: &str, out: &mut Vec<Symbol>) {
    let push = |name: &str, span: Span, out: &mut Vec<Symbol>| {
        out.push(Symbol {
            qualified: format!("{prefix}{name}"),
            simple: name.to_string(),
            location: Location {
                uri: uri.clone(),
                range: span_to_range(content, span),
            },
        });
    };

    for f in &module.functions {
        push(&f.name.name, f.name.span, out);
    }
    for s in &module.struct_defs {
        push(&s.name.name, s.name.span, out);
    }
    for g in &module.globals {
        push(&g.name.name, g.name.span, out);
    }
    for i in &module.inlines {
        push(&i.name.name, i.name.span, out);
    }
    for nm in &module.modules {
        push(&nm.name.name, nm.name.span, out);
        let inner = format!("{prefix}{}::", nm.name.name);
        collect(&nm.module, &inner, uri, content, out);
    }
}

/// Resolve `word` against the index, most-specific match first.
fn resolve(word: &str, index: &[Symbol]) -> Vec<Location> {
    // 1. exact fully-qualified match (`io::print`).
    let exact: Vec<_> = index.iter().filter(|s| s.qualified == word).collect();
    if !exact.is_empty() {
        return dedup(exact);
    }
    // 2. `word` is a trailing qualification of a fully-qualified name.
    let suffix = format!("::{word}");
    let by_suffix: Vec<_> = index.iter().filter(|s| s.qualified.ends_with(&suffix)).collect();
    if !by_suffix.is_empty() {
        return dedup(by_suffix);
    }
    // 3. bare name match (last `::` segment of `word`).
    let simple = word.rsplit("::").next().unwrap_or(word);
    dedup(index.iter().filter(|s| s.simple == simple).collect())
}

fn dedup(mut syms: Vec<&Symbol>) -> Vec<Location> {
    let mut out: Vec<Location> = Vec::new();
    syms.sort_by(|a, b| {
        (a.location.uri.as_str(), a.location.range.start.line)
            .cmp(&(b.location.uri.as_str(), b.location.range.start.line))
    });
    for s in syms {
        if !out.iter().any(|l| l.uri == s.location.uri && l.range == s.location.range) {
            out.push(s.location.clone());
        }
    }
    out
}

fn span_to_range(content: &str, span: Span) -> Range {
    Range {
        start: crate::byte_to_position(content, span.start()),
        end: crate::byte_to_position(content, span.end()),
    }
}

/// Convert an LSP [`Position`] (line, UTF-16 column) to a byte offset.
fn position_to_byte(text: &str, pos: Position) -> usize {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in text.char_indices() {
        if line == pos.line && col >= pos.character {
            return i;
        }
        if line > pos.line {
            return i;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
    }
    text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_same_file_function() {
        let src = "fn helper -> { }\nfn main -> { helper }";
        let uri = Url::parse("mem://test.tl").unwrap();
        // Cursor on the `helper` call (line 1, col 13).
        let locs = goto(&uri, src, Position { line: 1, character: 13 });
        assert_eq!(locs.len(), 1, "expected one definition, got {locs:?}");
        assert_eq!(locs[0].range.start.line, 0);
        assert_eq!(locs[0].range.start.character, 3); // `fn helper` -> name at col 3
    }

    #[test]
    fn resolves_qualified_module_member() {
        let src = "mod io { fn print -> { } }\nfn main -> { io::print }";
        let uri = Url::parse("mem://test.tl").unwrap();
        // Cursor on `io::print` (line 1). `io::print` starts at col 13.
        let locs = goto(&uri, src, Position { line: 1, character: 15 });
        assert_eq!(locs.len(), 1, "expected one definition, got {locs:?}");
        assert_eq!(locs[0].range.start.line, 0);
    }

    #[test]
    fn no_match_returns_empty() {
        let src = "fn main -> { }";
        let uri = Url::parse("mem://test.tl").unwrap();
        // Cursor on the space between `fn` and `main` (not an identifier).
        assert!(goto(&uri, src, Position { line: 0, character: 2 }).is_empty());
    }
}
