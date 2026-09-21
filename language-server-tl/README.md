# language-server-tl

A [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
implementation for **ToastLang (`.tl`)**, the stack-based concatenative language
that compiles to ToastASM / ToastCPU (see the repo root and `../compiler`).

It is built on [`tower-lsp`](https://crates.io/crates/tower-lsp) and reuses the
compiler's frontend directly: the `tlc` crate (`../compiler`) is exposed as a
library, so parsing, `#include` preprocessing, name resolution and the
type/stack-effect checker are **shared with the compiler and never drift from
it**. The LSP consumes `tlc::analyze()` and the `tlc::Diagnostic` type rather
than reimplementing any of the frontend.

## Architecture

```
source text
   │
   ▼
tlc::analyze(src, include_paths)   ← shared frontend (../compiler/src/lib.rs)
   │   preprocess → parse → check
   ▼
Analysis { ast: Option<Module>, diagnostics: Vec<Diagnostic> }
   │
   ▼
language-server-tl  ← maps AST + diagnostics onto LSP messages
```

Frontend API currently available to the LSP (`tlc::…`):

- `analyze(src, include_paths) -> Analysis` — the primary entry point.
- `parse(src) -> (Option<Module>, Vec<Diagnostic>)` — lex + parse, error recovery preserved.
- `check(&Module) -> Vec<Diagnostic>` — name resolution + type/stack-effect check.
- `line_col(src, byte_offset) -> (line, col)` — span → editor position conversion.
- `Diagnostic { start, end, severity, kind, message }` — byte-offset spans.

## Status

| Milestone | Feature | State |
|-----------|---------|-------|
| 0 | Shared-frontend refactor + Cargo workspace | ✅ done |
| 1 | **Diagnostics** (live parse/type errors) | ✅ done |
| 1 | **Semantic highlighting** (`semanticTokens`) | ✅ done |

Milestone 1 notes:

- **Semantic tokens** combine a lexical layer (`tlc::lex`) with an AST layer
  (`tlc::parse`) so identifiers are classified by their real role — functions,
  structs, modules, `let` locals, type names, `$a` type parameters, struct
  fields — and highlighting still degrades to the lexical layer when a file does
  not parse. Implemented in `src/semantic.rs`.
- **Diagnostics**: lex/parse errors are always published with correct positions.
  Type/stack-effect errors are published **only for files without `#include<…>`**
  — the preprocessor expands includes in place, which shifts byte offsets so
  checker spans would otherwise land in the wrong location. Lifting this is the
  first item under *Known frontend follow-ups*.

Everything below is the wishlist — not yet implemented.

## Wishlist

Ordered roughly by planned priority. Items marked **★** are the features that
are unusually valuable *because* `tl` is a statically-typed stack language and
generic LSPs cannot offer them.

### Tier 1 — core editor features
- [ ] **Hover** — show a word's type signature (stack effect) and its doc comment.
- [ ] **Go-to-definition** — jump to `fn` / `struct` / `global` / `inline` /
      local `let`, following `#include` across files.
- [ ] **Find references** (`textDocument/references`).
- [ ] **Completion** — words in scope, struct fields after `.`, module members
      after `::`, keywords.
- [ ] **Document symbols** (`documentSymbol`) — outline of fns/structs/globals/mods.
- [ ] **Rename** (`rename`) — workspace-wide, include-aware.

### Tier 2 — high value, moderate effort
- [ ] **Workspace symbols** (`workspace/symbol`) — fuzzy jump to any fn/struct.
- [ ] **Signature help** — show a call's stack effect and current argument.
- [ ] **Formatting** — a canonical pretty-printer for `.tl`.
- [ ] **Folding ranges**, **selection ranges**, **document highlight**.

### Tier 3 — ★ killer features unique to a typed stack language
- [ ] **★ Stack-state hints** — inlay hints / hover showing the *stack contents*
      at the cursor (e.g. `‹u16 u16 bool›`), or before/after each word. Tracking
      "what is on the stack right now" is the hardest part of writing
      concatenative code; the type checker already computes this.
- [ ] **★ Typed holes (`??`)** — the grammar already has a `HOLE` token. When the
      checker reaches `??`, report the expected type and the current stack, like
      Haskell/Idris typed holes.
- [ ] **★ Rich stack-effect mismatch diagnostics** — show declared vs. actual net
      stack effect, not just "type error".
- [ ] **Inlay hints for generics/casts** — show what `$a` resolved to at a call
      site, or the inferred type of a `let` binding.

### Semantic-token plan
Because the LSP has real type info, semantic tokens can distinguish things a
TextMate grammar (`../vscode-syntax-tl`) cannot:

- **Types:** `function`, `type`/`struct`, `namespace` (modules), `variable`
  (globals & `let` locals), `parameter` (`$a` generics), `keyword`, `operator`,
  `number`, `string`, `comment`.
- **Modifiers:** `declaration` (defining site), `readonly`/`static` (globals,
  inlines), `defaultLibrary` (std-lib words such as `io::print`).

## Known frontend follow-ups (needed to make the above great)

- [ ] **Multiple type-check diagnostics per file.** The type/stack-effect check
      is currently interwoven with codegen (`emit::module::emit_root_module`),
      which returns a *single* `(Span, String)` error. So today the LSP surfaces
      at most **one** `TypeCheck` diagnostic per run (lex/parse diagnostics are
      genuinely multiple). Refactoring the checker to accumulate all type errors
      with spans is a prerequisite for good diagnostics UX.
- [ ] **Incremental / cached analysis.** Re-analyzing the whole document (and its
      includes) on every keystroke is fine to start, but a document store with a
      workspace include-graph and caching will be needed for responsiveness.
- [ ] **Robust error recovery.** Lean on grmtools recovery so a half-typed file
      still yields a partial AST + diagnostics for all read-only features.

## Development

This crate is a member of the root Cargo workspace.

```sh
# from the repo root
cargo build -p language-server-tl
cargo test  -p language-server-tl
```
