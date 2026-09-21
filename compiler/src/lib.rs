//! ToastLang compiler frontend, exposed as a library.
//!
//! This crate provides the reusable *frontend* of the ToastLang compiler so
//! that both the `tlc` binary and the language server (`language-server-tl`)
//! can share the same lexer, parser, preprocessor, and type / stack-effect
//! checker.
//!
//! The high level entry points ([`parse`], [`check`], and [`analyze`]) collect
//! problems into a returned [`Diagnostic`] list (each carrying a source span)
//! instead of printing to stderr or exiting the process, which makes them
//! usable from an editor/LSP context. The `tlc` binary keeps a thin wrapper
//! (in `main.rs`) that reproduces the historical print-and-exit CLI behavior on
//! top of this API.

use lrlex::lrlex_mod;
use lrlex::LexerDef;
use lrpar::{lrpar_mod, LexParseError, Lexeme, Lexer};
use once_cell::sync::OnceCell;
use std::collections::HashSet;

pub mod emit;
pub mod parser_util;
pub mod preprocess;
pub mod util;

// The grmtools-generated lexer (`tl.l`) and parser (`tl.y`). These live at the
// crate root so the grammar's `use crate::parser_util::*` footer resolves.
lrlex_mod!("tl.l");
lrpar_mod!("tl.y");

pub use parser_util::types::Module;
/// Source span type used throughout the public AST (re-exported from `lrpar`).
pub use lrpar::Span;

static VERBOSE: OnceCell<bool> = OnceCell::new();

/// Set the global verbose flag. Safe to call more than once (later calls after
/// the first are ignored) so it does not panic when driven from an LSP.
pub fn set_verbose(verbose: bool) {
    let _ = VERBOSE.set(verbose);
}

/// Whether verbose logging is enabled.
pub fn is_verbose() -> bool {
    *VERBOSE.get().unwrap_or(&false)
}

/// Severity of a [`Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Which frontend phase produced a [`Diagnostic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// Lexing error (an unmatched / unexpected token).
    Lex,
    /// Parsing error (grammar mismatch); the parser recovers where possible.
    Parse,
    /// `#include` preprocessing / include-resolution error.
    Preprocess,
    /// Name resolution or type / stack-effect checking error.
    TypeCheck,
}

/// A problem found while analyzing source text.
///
/// [`start`](Diagnostic::start) and [`end`](Diagnostic::end) are byte offsets
/// into the *preprocessed* source text (the same text passed to [`parse`] /
/// [`check`]). Use [`line_col`] / [`Diagnostic::range_line_col`] to convert
/// them to 1-based line / 0-based column pairs for an editor.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Inclusive start byte offset into the (preprocessed) source.
    pub start: usize,
    /// Exclusive end byte offset into the (preprocessed) source.
    pub end: usize,
    pub severity: Severity,
    pub kind: DiagnosticKind,
    pub message: String,
}

impl Diagnostic {
    pub fn error(kind: DiagnosticKind, start: usize, end: usize, message: String) -> Self {
        Diagnostic {
            start,
            end,
            severity: Severity::Error,
            kind,
            message,
        }
    }

    /// Convert this diagnostic's byte span to `((start_line, start_col),
    /// (end_line, end_col))`, where lines are 1-based and columns 0-based.
    pub fn range_line_col(&self, source: &str) -> ((usize, usize), (usize, usize)) {
        (
            line_col(source, self.start),
            line_col(source, self.end),
        )
    }
}

/// Convert a byte offset into `source` to a `(line, column)` pair, where the
/// line is 1-based and the column is 0-based (measured in `char`s).
pub fn line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let offset = byte_offset.min(source.len());
    let mut line = 1usize;
    let mut col = 0usize;
    for (i, c) in source.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// A lexical category for a [`Token`], used to drive semantic highlighting.
///
/// This is a coarse, lexer-level classification (it cannot, on its own, tell a
/// function name from a struct name — both are [`TokenKind::Ident`]). Consumers
/// that want the finer distinction combine this with the parsed AST, upgrading
/// identifier spans to their true role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// A language keyword (`fn`, `let`, `if`, `struct`, `mod`, `as`, ...).
    Keyword,
    /// A boolean literal (`true` / `false`).
    Bool,
    /// An integer literal (decimal, hex or binary).
    Number,
    /// A string literal.
    Str,
    /// A character literal.
    Char,
    /// An operator (`+`, `<<`, `||`, `??`, `*`, `$`, ...).
    Operator,
    /// A struct field access (`.field`).
    StructAccess,
    /// A bare identifier (word / function call / type name / variable).
    Ident,
    /// Bracketing / grouping punctuation (`(` `)` `[` `]` `{` `}`).
    Punct,
    /// Anything the lexer could not match.
    Unknown,
}

/// A lexed token: a source span plus its lexical [`TokenKind`].
#[derive(Debug, Clone, Copy)]
pub struct Token {
    /// Inclusive start byte offset into `source`.
    pub start: usize,
    /// Exclusive end byte offset into `source`.
    pub end: usize,
    pub kind: TokenKind,
}

/// Classify a lexer rule name (as declared in `tl.l`) into a [`TokenKind`].
fn classify_token_name(name: &str) -> TokenKind {
    match name {
        "GLOBAL" | "INLINE" | "FN" | "LET" | "STRUCT" | "MOD" | "USING" | "UNROLL" | "IF"
        | "ELSE" | "WHILE" | "PTR_OP" | "AS" | "SIZEOF" | "RETURN" | "RARROW" => TokenKind::Keyword,
        "TRUE_BOOL" | "FALSE_BOOL" => TokenKind::Bool,
        "DEC_INT" | "HEX_INT" | "BIN_INT" => TokenKind::Number,
        "STRING" => TokenKind::Str,
        "CHAR" => TokenKind::Char,
        "LOR" | "LAND" | "LNOT" | "INCR" | "DECR" | "ADD" | "SUB" | "BOR" | "BAND" | "BNOT"
        | "SSHR" | "SHR" | "SHL" | "XOR" | "EQ" | "LT" | "LTE" | "GT" | "GTE" | "HOLE"
        | "GENERIC" | "PTR" => TokenKind::Operator,
        "STRUCT_ACCESS" => TokenKind::StructAccess,
        "IDENT" => TokenKind::Ident,
        "LP" | "RP" | "LS" | "RS" | "LB" | "RB" => TokenKind::Punct,
        _ => TokenKind::Unknown,
    }
}

/// Lex `source` into a flat list of classified [`Token`]s, in source order.
///
/// This reuses the compiler's generated lexer (`tl.l`) so token boundaries and
/// classification stay in lock-step with the language. Comments and whitespace
/// are skipped by the lexer and so are not represented here. Faulty (error)
/// lexemes are reported as [`TokenKind::Unknown`] rather than dropped, so a
/// broken file still highlights the parts around the error.
pub fn lex(source: &str) -> Vec<Token> {
    let lexer_def = tl_l::lexerdef();
    let lexer = lexer_def.lexer(source);
    let mut tokens = Vec::new();
    for lexeme in lexer.iter() {
        match lexeme {
            Ok(l) => {
                let span = l.span();
                // Map the lexeme's token id back to its `tl.l` rule name, then
                // classify. Lexemes always come from a named rule, so a missing
                // name falls back to `Unknown`.
                let kind = lexer_def
                    .get_rule_by_id(l.tok_id())
                    .name
                    .as_deref()
                    .map(classify_token_name)
                    .unwrap_or(TokenKind::Unknown);
                if kind != TokenKind::Punct {
                    tokens.push(Token {
                        start: span.start(),
                        end: span.end(),
                        kind,
                    });
                }
            }
            Err(e) => {
                let span = e.span();
                tokens.push(Token {
                    start: span.start(),
                    end: span.end(),
                    kind: TokenKind::Unknown,
                });
            }
        }
    }

    tokens
}

/// Result of running the full frontend over a source file.
#[derive(Debug)]
pub struct Analysis {
    /// The parsed AST, if a (possibly partial) parse succeeded.
    pub ast: Option<Module>,
    /// All diagnostics gathered across preprocessing, parsing and checking.
    pub diagnostics: Vec<Diagnostic>,
}

/// Run the `#include` preprocessor over `source`.
///
/// Returns the expanded source, or a single [`Diagnostic`] describing the
/// include-resolution failure. `include_paths` is a list of base directories
/// searched (in order) for `#include<name>` directives.
pub fn preprocess_source(
    source: &str,
    include_paths: &[String],
) -> Result<String, Diagnostic> {
    preprocess::preprocess(source, &include_paths.to_vec(), &mut HashSet::new())
        .map_err(|msg| Diagnostic::error(DiagnosticKind::Preprocess, 0, 0, msg))
}

/// Lex and parse `source` into an AST, collecting all lexer/parser diagnostics.
///
/// grmtools' error recovery is preserved, so a partially broken file can still
/// yield a partial [`Module`] alongside the diagnostics describing the errors.
pub fn parse(source: &str) -> (Option<Module>, Vec<Diagnostic>) {
    let lexer_def = tl_l::lexerdef();
    let lexer = lexer_def.lexer(source);
    let (res, errs) = tl_y::parse(&lexer);

    let mut diagnostics = Vec::new();
    for e in errs {
        let message = e.pp(&lexer, &tl_y::token_epp);
        let (start, end, kind) = match &e {
            LexParseError::LexError(le) => {
                let span = le.span();
                (span.start(), span.end(), DiagnosticKind::Lex)
            }
            LexParseError::ParseError(pe) => {
                let span = pe.lexeme().span();
                (span.start(), span.end(), DiagnosticKind::Parse)
            }
        };
        diagnostics.push(Diagnostic::error(kind, start, end, message));
    }

    // `res` is `Option<Result<Module, ()>>`: a full parse yields `Some(Ok(..))`,
    // while both `None` and `Some(Err(..))` indicate the AST could not be built.
    let ast = match res {
        Some(Ok(module)) => Some(module),
        _ => None,
    };

    (ast, diagnostics)
}

/// Run name resolution + the type / stack-effect checker over `module`.
///
/// Returns the diagnostics found (empty when the module checks cleanly). The
/// underlying pass currently short-circuits on the first checking error, so at
/// most one [`DiagnosticKind::TypeCheck`] diagnostic is returned.
pub fn check(module: &Module) -> Vec<Diagnostic> {
    match emit::module::emit_root_module(module) {
        Ok(_) => Vec::new(),
        Err((span, message)) => vec![Diagnostic::error(
            DiagnosticKind::TypeCheck,
            span.start(),
            span.end(),
            message,
        )],
    }
}

/// A diagnostic whose location has been mapped back to a specific source file.
///
/// Unlike [`Diagnostic`] (whose offsets are into preprocessed text), the offsets
/// here are into the original `file` the problem lives in — which may be the
/// document being edited or one of its `#include`d files.
#[derive(Debug, Clone)]
pub struct MappedDiagnostic {
    /// Path of the file this diagnostic is located in.
    pub file: String,
    /// Byte offset range within `file`.
    pub start: usize,
    pub end: usize,
    pub severity: Severity,
    pub kind: DiagnosticKind,
    pub message: String,
}

/// Run the type / stack-effect checker over `source` *with `#include`s expanded*
/// and map every diagnostic back to the real file and byte offset it belongs to.
///
/// `root_path` is the path of `source` itself (used both to resolve relative
/// includes' provenance and to let callers tell which diagnostics belong to the
/// document being edited). `include_paths` is searched for `#include<name>`.
///
/// This is what lets the language server surface type errors on files that use
/// `#include` — the plain [`analyze`]/[`check`] path reports offsets into the
/// expanded text, which do not line up with the file on disk.
///
/// Note: like the checker it wraps, this currently yields at most one
/// type-check diagnostic per call. If preprocessing fails (an include cannot be
/// resolved) a single [`DiagnosticKind::Preprocess`] diagnostic is returned,
/// located at the offending `#include` in `root_path` when known.
pub fn check_mapped(
    source: &str,
    root_path: &str,
    include_paths: &[String],
) -> Vec<MappedDiagnostic> {
    let (preprocessed, map) = match preprocess::preprocess_mapped(source, root_path, include_paths) {
        Ok(pair) => pair,
        Err(e) => {
            let (start, end) = e.root_span.unwrap_or((0, 0));
            return vec![MappedDiagnostic {
                file: root_path.to_string(),
                start,
                end,
                severity: Severity::Error,
                kind: DiagnosticKind::Preprocess,
                message: e.message,
            }];
        }
    };

    let (ast, _parse_diags) = parse(&preprocessed);
    let module = match ast {
        Some(m) => m,
        // Parse errors are reported separately (against the raw document, where
        // their offsets are correct), so don't duplicate them here.
        None => return Vec::new(),
    };

    let mut out = Vec::new();
    for d in check(&module) {
        if let Some((file, start, end)) = map.map_range(d.start, d.end) {
            out.push(MappedDiagnostic {
                file: file.to_string(),
                start,
                end,
                severity: d.severity,
                kind: d.kind,
                message: d.message,
            });
        }
    }
    out
}

/// Full frontend pipeline: preprocess -> parse -> check.
///
/// Given source text and a list of include base paths, returns the (possibly
/// partial) AST together with every diagnostic gathered along the way. This is
/// the primary entry point for the language server.
pub fn analyze(source: &str, include_paths: &[String]) -> Analysis {
    let preprocessed = match preprocess_source(source, include_paths) {
        Ok(text) => text,
        Err(diag) => {
            return Analysis {
                ast: None,
                diagnostics: vec![diag],
            }
        }
    };

    let (ast, mut diagnostics) = parse(&preprocessed);
    if let Some(module) = &ast {
        diagnostics.extend(check(module));
    }

    Analysis { ast, diagnostics }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_program_parses_without_diagnostics() {
        let (ast, diags) = parse("fn main -> { 1 2 + }");
        assert!(ast.is_some(), "expected a parsed AST");
        assert!(
            diags.is_empty(),
            "valid program should produce no parse diagnostics, got {diags:?}"
        );
    }

    #[test]
    fn broken_program_yields_diagnostics_with_spans() {
        // `@` is an unmatched character -> at least one diagnostic with a span.
        let (_ast, diags) = parse("fn main -> { @ }");
        assert!(!diags.is_empty(), "expected at least one diagnostic");
        let d = &diags[0];
        assert!(d.end >= d.start, "span must be well-formed");
        assert_eq!(d.severity, Severity::Error);
    }

    #[test]
    fn analyze_reports_type_check_errors() {
        // `+` with an empty stack is a stack-effect error caught by the checker.
        let analysis = analyze("fn main -> { + }", &[]);
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|d| d.kind == DiagnosticKind::TypeCheck),
            "expected a type-check diagnostic, got {:?}",
            analysis.diagnostics
        );
    }

    #[test]
    fn lex_classifies_tokens() {
        let toks = lex("fn main -> { 1 \"hi\" + }");
        // First token is the `fn` keyword.
        assert_eq!(toks[0].kind, TokenKind::Keyword);
        // The identifier `main` is present.
        assert!(toks.iter().any(|t| t.kind == TokenKind::Ident));
        assert!(toks.iter().any(|t| t.kind == TokenKind::Number));
        assert!(toks.iter().any(|t| t.kind == TokenKind::Str));
        assert!(toks.iter().any(|t| t.kind == TokenKind::Operator));
        // Braces are punctuation and are intentionally not emitted.
        assert!(toks.iter().all(|t| t.kind != TokenKind::Punct));
    }

    #[test]
    fn check_mapped_without_includes_is_identity() {
        let src = "fn main -> { + }";
        let plus = src.find('+').unwrap();
        let diags = check_mapped(src, "mem://main.tl", &[]);
        let td = diags
            .iter()
            .find(|d| d.kind == DiagnosticKind::TypeCheck)
            .expect("expected a type-check diagnostic");
        assert_eq!(td.file, "mem://main.tl");
        assert_eq!(td.start, plus, "offset should be unchanged without includes");
    }

    #[test]
    fn check_mapped_maps_type_error_past_include() {
        use std::fs;
        let dir = std::env::temp_dir().join("tlc_test_map_past_include");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("lib/std")).unwrap();
        // A larger include, so the preprocessed offset of `+` differs a lot from
        // its offset in the original file.
        fs::write(
            dir.join("lib/std/helper.tl"),
            "mod helper {\n  fn a -> { }\n  fn b -> { }\n  fn c -> { }\n}\n",
        )
        .unwrap();

        let root_path = dir.join("main.tl").to_string_lossy().to_string();
        let source = "#include<std/helper>\nfn main -> { + }\n";
        let plus_off = source.find('+').unwrap();
        let include_paths = vec![dir.join("lib").to_string_lossy().to_string()];

        let diags = check_mapped(source, &root_path, &include_paths);
        let td = diags
            .iter()
            .find(|d| d.kind == DiagnosticKind::TypeCheck)
            .expect("expected a mapped type-check diagnostic");
        assert_eq!(td.file, root_path);
        assert_eq!(
            td.start, plus_off,
            "diagnostic must map back to the `+` in the original file, not the expanded offset"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_mapped_reports_missing_include() {
        let src = "#include<does/not/exist>\nfn main -> { }";
        let diags = check_mapped(src, "main.tl", &[]);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind, DiagnosticKind::Preprocess);
        assert_eq!(diags[0].start, 0, "should point at the include directive");
        assert!(diags[0].end > diags[0].start);
    }

    #[test]
    fn line_col_conversion() {
        let src = "ab\ncde";
        assert_eq!(line_col(src, 0), (1, 0));
        assert_eq!(line_col(src, 3), (2, 0)); // first char of second line
        assert_eq!(line_col(src, 4), (2, 1));
    }
}

/// Compile `module` all the way to ToastASM (`tasm`) text.
///
/// This is a backend concern that the LSP does not need, but it is exposed so
/// the binary can share the code path. On a checking/emission error it returns
/// the corresponding [`Diagnostic`].
pub fn emit_tasm(module: &Module) -> Result<String, Diagnostic> {
    emit::module::emit_root_module(module).map_err(|(span, message)| {
        Diagnostic::error(
            DiagnosticKind::TypeCheck,
            span.start(),
            span.end(),
            message,
        )
    })
}
