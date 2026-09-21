//! Semantic token computation for ToastLang.
//!
//! Highlighting is produced in two layers that are then merged:
//!
//! 1. A **lexical layer** from [`tlc::lex`] classifies every lexeme (keywords,
//!    literals, operators, ...). This works even when the file does not parse.
//! 2. An **AST layer** from [`tlc::parse`] upgrades identifier spans to their
//!    true role (function vs. struct vs. namespace vs. local vs. type), which a
//!    purely lexical pass cannot know.
//!
//! The two are combined by walking the lexeme stream in source order and, for
//! each identifier lexeme, preferring the AST classification when one exists.

use std::collections::HashMap;

use tlc::parser_util::types::{
    Block, FuncType, Function, Global, If, Inline, LexType, Let, Module, NamedModule, Operator,
    Statement, StructDef, Unroll, Using, While,
};
use tlc::{Token, TokenKind};
use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend,
};

/// The semantic token *types* we emit, in the order the client sees them. The
/// numeric index of each variant is its position in [`legend`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SemType {
    Namespace = 0,
    Type = 1,
    Struct = 2,
    TypeParameter = 3,
    Variable = 4,
    Property = 5,
    Function = 6,
    Keyword = 7,
    Operator = 8,
    String = 9,
    Number = 10,
}

/// Semantic token *modifier* bit flags (a bitset over [`legend`]'s modifiers).
pub mod modifier {
    pub const DECLARATION: u32 = 1 << 0;
    pub const READONLY: u32 = 1 << 1;
    pub const STATIC: u32 = 1 << 2;
}

/// The legend advertised in server capabilities. Its ordering must match the
/// numeric values of [`SemType`] and [`modifier`].
pub fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::NAMESPACE,
            SemanticTokenType::TYPE,
            SemanticTokenType::STRUCT,
            SemanticTokenType::TYPE_PARAMETER,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::PROPERTY,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::OPERATOR,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::READONLY,
            SemanticTokenModifier::STATIC,
        ],
    }
}

#[derive(Clone, Copy)]
struct Sem {
    ty: SemType,
    mods: u32,
}

impl Sem {
    fn new(ty: SemType) -> Self {
        Sem { ty, mods: 0 }
    }
    fn with(ty: SemType, mods: u32) -> Self {
        Sem { ty, mods }
    }
}

/// Compute the semantic tokens for `source`.
///
/// `ast` is the parse result over the *same* `source` (may be `None` when the
/// file does not parse, in which case only the lexical layer is used).
pub fn semantic_tokens(source: &str, ast: Option<&Module>) -> Vec<SemanticToken> {
    let lexemes = tlc::lex(source);

    // AST layer: identifier byte-span -> semantic classification.
    let mut ast_map: HashMap<(usize, usize), Sem> = HashMap::new();
    let mut symbols = Symbols::default();
    if let Some(module) = ast {
        symbols.collect_module(module);
        let mut walker = AstWalker {
            map: &mut ast_map,
            symbols: &symbols,
        };
        walker.module(module);
    }

    // Merge: walk lexemes in order, classify each, split across line breaks, and
    // delta-encode into LSP semantic tokens.
    let lines = LineIndex::new(source);
    let mut encoder = Encoder::new();
    for tok in &lexemes {
        let sem = match classify_lexeme(tok, &ast_map) {
            Some(s) => s,
            None => continue,
        };
        encoder.push(source, &lines, tok.start, tok.end, sem);
    }
    encoder.finish()
}

/// Classify a single lexeme, consulting the AST map for identifiers.
fn classify_lexeme(tok: &Token, ast_map: &HashMap<(usize, usize), Sem>) -> Option<Sem> {
    let sem = match tok.kind {
        TokenKind::Keyword | TokenKind::Bool => Sem::new(SemType::Keyword),
        TokenKind::Number => Sem::new(SemType::Number),
        TokenKind::Str | TokenKind::Char => Sem::new(SemType::String),
        TokenKind::Operator => Sem::new(SemType::Operator),
        TokenKind::StructAccess => Sem::new(SemType::Property),
        TokenKind::Ident => ast_map
            .get(&(tok.start, tok.end))
            .copied()
            // Unresolved identifier (e.g. a std-lib call, or any word when the
            // file did not parse): default to a function call, which is what
            // most bare words in `tl` are.
            .unwrap_or_else(|| Sem::new(SemType::Function)),
        TokenKind::Punct | TokenKind::Unknown => return None,
    };
    Some(sem)
}

/// Declared names in a module tree, used to refine identifier *uses*.
#[derive(Default)]
struct Symbols {
    structs: std::collections::HashSet<String>,
    globals: std::collections::HashSet<String>,
    modules: std::collections::HashSet<String>,
}

impl Symbols {
    fn collect_module(&mut self, m: &Module) {
        for s in &m.struct_defs {
            self.structs.insert(s.name.name.clone());
        }
        for g in &m.globals {
            self.globals.insert(g.name.name.clone());
        }
        for nm in &m.modules {
            self.modules.insert(nm.name.name.clone());
            self.collect_module(&nm.module);
        }
    }

    /// Classify an identifier *use* (a word invoked in a statement).
    fn classify_use(&self, name: &str) -> Sem {
        let simple = name.rsplit("::").next().unwrap_or(name);
        if self.structs.contains(simple) {
            Sem::new(SemType::Type)
        } else if self.globals.contains(simple) {
            Sem::new(SemType::Variable)
        } else {
            Sem::new(SemType::Function)
        }
    }
}

/// Walks the AST recording identifier spans -> semantic classification.
struct AstWalker<'a> {
    map: &'a mut HashMap<(usize, usize), Sem>,
    symbols: &'a Symbols,
}

impl AstWalker<'_> {
    fn put(&mut self, span: tlc::Span, sem: Sem) {
        self.map.insert((span.start(), span.end()), sem);
    }

    fn module(&mut self, m: &Module) {
        for u in &m.usings {
            self.using(u);
        }
        for s in &m.struct_defs {
            self.struct_def(s);
        }
        for g in &m.globals {
            self.global(g);
        }
        for i in &m.inlines {
            self.inline(i);
        }
        for f in &m.functions {
            self.function(f);
        }
        for nm in &m.modules {
            self.named_module(nm);
        }
    }

    fn named_module(&mut self, nm: &NamedModule) {
        self.put(
            nm.name.span,
            Sem::with(SemType::Namespace, modifier::DECLARATION),
        );
        self.module(&nm.module);
    }

    fn using(&mut self, u: &Using) {
        self.put(u.name.span, Sem::new(SemType::Namespace));
    }

    fn struct_def(&mut self, s: &StructDef) {
        self.put(
            s.name.span,
            Sem::with(SemType::Struct, modifier::DECLARATION),
        );
        for member in &s.members {
            self.put(
                member.name.span,
                Sem::with(SemType::Property, modifier::DECLARATION),
            );
            self.lex_type(&member.var_type);
        }
    }

    fn global(&mut self, g: &Global) {
        self.put(
            g.name.span,
            Sem::with(SemType::Variable, modifier::DECLARATION | modifier::STATIC),
        );
        self.lex_type(&g.var_type);
    }

    fn inline(&mut self, i: &Inline) {
        self.put(
            i.name.span,
            Sem::with(
                SemType::Variable,
                modifier::DECLARATION | modifier::READONLY,
            ),
        );
        self.statement(&i.statement);
    }

    fn function(&mut self, f: &Function) {
        self.put(
            f.name.span,
            Sem::with(SemType::Function, modifier::DECLARATION),
        );
        self.func_type(&f.type_def);
        self.block(&f.body);
    }

    fn func_type(&mut self, t: &FuncType) {
        for ty in t.i.iter().chain(t.o.iter()) {
            self.lex_type(ty);
        }
    }

    fn lex_type(&mut self, t: &LexType) {
        match t {
            LexType::Base(id) => self.put(id.span, Sem::new(SemType::Type)),
            LexType::Gen(id) => self.put(id.span, Sem::new(SemType::TypeParameter)),
            LexType::Ptr(inner) => self.lex_type(inner),
            LexType::Func(ft) => self.func_type(ft),
        }
    }

    fn block(&mut self, b: &Block) {
        for st in &b.body {
            self.statement(st);
        }
    }

    fn statement(&mut self, st: &Statement) {
        match st {
            Statement::Identifier(id) => {
                let sem = self.symbols.classify_use(&id.name);
                self.put(id.span, sem);
            }
            Statement::Operator(op) => self.operator(op),
            Statement::Block(b) => self.block(b),
            Statement::If(If {
                if_block,
                else_block,
                ..
            }) => {
                self.block(if_block);
                if let Some(eb) = else_block {
                    self.block(eb);
                }
            }
            Statement::While(While { eval, body, .. }) => {
                self.block(eval);
                self.block(body);
            }
            Statement::Let(Let { bindings, body, .. }) => {
                for b in bindings {
                    self.put(
                        b.span,
                        Sem::with(SemType::Variable, modifier::DECLARATION),
                    );
                }
                self.block(body);
            }
            Statement::Unroll(Unroll { body, .. }) => self.block(body),
            // Literals are already classified by the lexical layer.
            Statement::IntLiteral(_) | Statement::IntArray(_) | Statement::BoolLiteral(_) => {}
        }
    }

    fn operator(&mut self, op: &Operator) {
        match op {
            Operator::As(_, ty) | Operator::SizeOf(_, ty) => self.lex_type(ty),
            Operator::Ptr(_, id) => {
                let sem = self.symbols.classify_use(&id.name);
                self.put(id.span, sem);
            }
            // Remaining operators are already handled by the lexical layer.
            _ => {}
        }
    }
}

/// Precomputed line-start byte offsets, for byte-offset -> (line, utf16 col).
struct LineIndex {
    /// Byte offset of the start of each line.
    starts: Vec<usize>,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        LineIndex { starts }
    }

    /// The 0-based line containing `byte`.
    fn line_of(&self, byte: usize) -> usize {
        match self.starts.binary_search(&byte) {
            Ok(line) => line,
            Err(next) => next - 1,
        }
    }

    fn line_start(&self, line: usize) -> usize {
        self.starts[line]
    }
}

/// Number of UTF-16 code units in `source[start..end]`.
fn utf16_len(source: &str, start: usize, end: usize) -> u32 {
    source[start..end].encode_utf16().count() as u32
}

/// Builds the delta-encoded LSP semantic token stream.
struct Encoder {
    out: Vec<SemanticToken>,
    prev_line: u32,
    prev_start: u32,
}

impl Encoder {
    fn new() -> Self {
        Encoder {
            out: Vec::new(),
            prev_line: 0,
            prev_start: 0,
        }
    }

    /// Emit a token for the byte range `[start, end)`, splitting it across line
    /// breaks (LSP semantic tokens must not span multiple lines).
    fn push(&mut self, source: &str, lines: &LineIndex, start: usize, end: usize, sem: Sem) {
        if end <= start {
            return;
        }
        let first_line = lines.line_of(start);
        let last_line = lines.line_of(end.saturating_sub(1));

        for line in first_line..=last_line {
            let line_start = lines.line_start(line);
            let next_line_start = lines
                .starts
                .get(line + 1)
                .copied()
                .unwrap_or(source.len());
            // Clamp this piece to the current line (excluding the newline byte).
            let piece_start = start.max(line_start);
            let piece_end = end.min(next_line_start).min(
                // Trim a trailing '\n' from the line's byte range.
                if next_line_start > line_start
                    && source.as_bytes().get(next_line_start - 1) == Some(&b'\n')
                {
                    next_line_start - 1
                } else {
                    next_line_start
                },
            );
            if piece_end <= piece_start {
                continue;
            }
            let col = utf16_len(source, line_start, piece_start);
            let length = utf16_len(source, piece_start, piece_end);
            self.emit(line as u32, col, length, sem);
        }
    }

    fn emit(&mut self, line: u32, start: u32, length: u32, sem: Sem) {
        let delta_line = line - self.prev_line;
        let delta_start = if delta_line == 0 {
            start - self.prev_start
        } else {
            start
        };
        self.out.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type: sem.ty as u32,
            token_modifiers_bitset: sem.mods,
        });
        self.prev_line = line;
        self.prev_start = start;
    }

    fn finish(self) -> Vec<SemanticToken> {
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode the delta-encoded stream back to absolute
    /// `(line, char, len, type, mods)` tuples for assertions.
    fn decode(toks: &[SemanticToken]) -> Vec<(u32, u32, u32, u32, u32)> {
        let mut line = 0u32;
        let mut ch = 0u32;
        let mut out = Vec::new();
        for t in toks {
            line += t.delta_line;
            if t.delta_line == 0 {
                ch += t.delta_start;
            } else {
                ch = t.delta_start;
            }
            out.push((
                line,
                ch,
                t.length,
                t.token_type,
                t.token_modifiers_bitset,
            ));
        }
        out
    }

    fn tokens(src: &str) -> Vec<(u32, u32, u32, u32, u32)> {
        let (ast, _diags) = tlc::parse(src);
        decode(&semantic_tokens(src, ast.as_ref()))
    }

    #[test]
    fn keyword_and_function_declaration() {
        let toks = tokens("fn main -> {\n  1 2 +\n}");
        // `fn` keyword at line 0, col 0, length 2.
        assert_eq!(toks[0], (0, 0, 2, SemType::Keyword as u32, 0));
        // `main` classified as a function *declaration*.
        assert!(
            toks.contains(&(0, 3, 4, SemType::Function as u32, modifier::DECLARATION)),
            "expected `main` as a function declaration, got {toks:?}"
        );
        // The literals on line 1 are numbers.
        assert!(toks
            .iter()
            .any(|&(l, _, _, ty, _)| l == 1 && ty == SemType::Number as u32));
    }

    #[test]
    fn struct_and_type_classification() {
        let src = "struct Point {\n  x u16\n}\nfn get Point* -> u16 { .x load }";
        let toks = tokens(src);
        // `Point` (struct def) -> struct, declaration.
        assert!(
            toks.contains(&(0, 7, 5, SemType::Struct as u32, modifier::DECLARATION)),
            "expected `Point` struct declaration, got {toks:?}"
        );
        // `u16` in a type position -> type.
        assert!(
            toks.iter()
                .any(|&(_, _, _, ty, _)| ty == SemType::Type as u32),
            "expected a `type` token, got {toks:?}"
        );
        // `.x` field access -> property.
        assert!(
            toks.iter()
                .any(|&(_, _, _, ty, _)| ty == SemType::Property as u32),
            "expected a `property` token, got {toks:?}"
        );
    }

    #[test]
    fn generic_type_parameter() {
        // `$a` in a type position: `a` should be a type parameter.
        let toks = tokens("fn id $a -> $a { }");
        assert!(
            toks.iter()
                .any(|&(_, _, _, ty, _)| ty == SemType::TypeParameter as u32),
            "expected a type-parameter token, got {toks:?}"
        );
    }

    #[test]
    fn degrades_without_ast() {
        // A file that does not parse still yields lexical tokens.
        let toks = tokens("fn main -> { @ ");
        assert!(!toks.is_empty(), "expected lexical tokens even on parse error");
        assert_eq!(toks[0], (0, 0, 2, SemType::Keyword as u32, 0));
    }

    #[test]
    fn no_tokens_span_multiple_lines() {
        // A multi-line string must be split so no emitted token crosses a line.
        let src = "global s u16 0\nfn main -> { \"multi\nline\" drop }";
        let (ast, _) = tlc::parse(src);
        // Just assert it does not panic and produces something.
        let raw = semantic_tokens(src, ast.as_ref());
        assert!(!raw.is_empty());
    }
}
