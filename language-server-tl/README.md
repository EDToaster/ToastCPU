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
| 1 | **Diagnostics** (live parse/type errors, include-aware) | ✅ done |
| 1 | **Semantic highlighting** (`semanticTokens`) | ✅ done |
| 2 | **Go-to-definition** (cross-file, top-level items) | ✅ done |
| 3 | **Stack inspection** (hover) + **typed holes** (`??`) | ✅ done |

Milestone 1 notes:

- **Semantic tokens** combine a lexical layer (`tlc::lex`) with an AST layer
  (`tlc::parse`) so identifiers are classified by their real role — functions,
  structs, modules, `let` locals, type names, `$a` type parameters, struct
  fields — and highlighting still degrades to the lexical layer when a file does
  not parse. Implemented in `src/semantic.rs`.
- **Diagnostics**: lex/parse errors come from the raw document (correct
  positions; `#include` lines lex as comments). Type/stack-effect errors come
  from an **include-expanded, source-mapped** analysis (`tlc::check_mapped`):
  the preprocessor now records provenance (`SourceMap`), so a checker span in
  the expanded program is mapped back to the real file and offset. Errors that
  map to the open document are shown at the right place; errors inside included
  files are left to those files. Include search paths mirror the compiler
  convention (`-I../../lib -I./`): the document's directory plus any ancestor
  `lib/`. Implemented in `src/main.rs` (`publish_diagnostics`, `include_paths_for`).
  - *Caveat*: a **library file opened on its own** may show errors it wouldn't as
    part of a program — e.g. `std/io.tl` reports `io::clear is already defined`
    because `mem.tl` includes `std/io`, so rooting the include graph at `io.tl`
    double-defines it. The compiler reports the same thing; such files simply
    aren't valid compilation roots.

Everything below is the wishlist — not yet implemented.

## Wishlist

Ordered roughly by planned priority. Items marked **★** are the features that
are unusually valuable *because* `tl` is a statically-typed stack language and
generic LSPs cannot offer them.

### Tier 1 — core editor features
- [ ] **Hover** — show a word's type signature (stack effect) and its doc comment.
- [x] **Go-to-definition** — jumps to a top-level `fn` / `struct` / `global` /
      `inline` / `mod`, following `#include` across files (`src/definition.rs`).
      *v1 scope*: when the cursor is on a definition's own name, it resolves to
      exactly that definition (by AST position). For *uses*, a module-qualified
      name (`io::print`) resolves exactly; a **bare** name defined in several
      modules returns all candidates — `using`/enclosing-module scoping is not
      applied yet. **Not yet** handled: local `let` bindings and struct-field
      (`.field`) targets.
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
- [x] **★ Stack-state inspection (hover)** — hovering a word shows the type stack
      at that point (`Stack: u16 u16`). The checker records the stack after each
      statement via `tlc::analyze_mapped`; the LSP surfaces it in `src/stack.rs`.
- [x] **★ Typed holes (`??`)** — the checker records the stack at every `??`; the
      LSP shows it both on hover (`Stack at ??: …`) and as an info diagnostic, so
      it's always visible. *(Previously `??` printed to stdout, which corrupted
      the LSP stream — fixed as part of this.)*
- [ ] **★ Stack-state inlay hints** — the same probe data rendered inline
      (between statements) instead of only on hover.
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

- [x] **Include-aware diagnostics.** Type errors now surface on files that use
      `#include`, via a provenance-tracking preprocessor (`SourceMap` /
      `preprocess_mapped`) and `tlc::check_mapped`, which map expanded offsets
      back to the real file.
- [ ] **Multiple type-check diagnostics per file.** The type/stack-effect check
      is currently interwoven with codegen (`emit::module::emit_root_module`),
      which returns a *single* `(Span, String)` error. So today the LSP surfaces
      at most **one** `TypeCheck` diagnostic per run (lex/parse diagnostics are
      genuinely multiple). Refactoring the checker to accumulate all type errors
      with spans is the next diagnostics-quality step.
- [ ] **Publish diagnostics into included files.** Errors that map to an
      `#include`d file are currently dropped from the open document's list; they
      could instead be published under that file's own URI.
- [ ] **Incremental / cached analysis.** Re-analyzing the whole document (and its
      includes, which are re-read from disk) on every keystroke is fine to start,
      but a document store with a workspace include-graph and caching will be
      needed for responsiveness.
- [ ] **Robust error recovery.** Lean on grmtools recovery so a half-typed file
      still yields a partial AST + diagnostics for all read-only features.

## Development

This crate is a member of the root Cargo workspace.

```sh
# from the repo root
cargo build -p language-server-tl
cargo test  -p language-server-tl
```
