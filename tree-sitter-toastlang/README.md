# tree-sitter-toastlang

A [tree-sitter](https://tree-sitter.github.io/) grammar for **ToastLang (`.tl`)**.

Its purpose is **syntax highlighting** in tree-sitter editors (Helix, Neovim,
GitHub, …). It is hand-translated from the compiler's real grammar
(`../compiler/src/tl.l` and `../compiler/src/tl.y`) — keep it in sync with those
when the language changes.

> Note: correctness-critical parsing lives in `tl.y` and is reused by the
> compiler and the language server (`../language-server-tl`). This grammar is
> *only* for highlighting, where a slightly-off token is cosmetic, not a bug.
> Where `tl.y` uses one `Identifier` token for every role, this grammar
> deliberately distinguishes them (`type_identifier`, `word`, declaration
> fields) so highlight queries can color them differently.

## Files

- `grammar.js` — the grammar definition (edit this).
- `queries/highlights.scm` — highlight queries (Helix capture scopes).
- `src/` — the generated parser (committed so editors can build without the CLI).

## Build

The tree-sitter CLI is a dev dependency:

```sh
npm install                      # fetches tree-sitter-cli
npx tree-sitter generate --abi 14   # regenerate src/ after editing grammar.js
npx tree-sitter build -o toastlang.so
```

Verify it parses every sample program:

```sh
for f in $(cd .. && git ls-files '*.tl'); do npx tree-sitter parse "../$f" >/dev/null || echo "FAIL $f"; done
```

## Install into Helix (via symlink)

Symlinking lets a rebuild here update Helix automatically. With Helix's runtime
at `$HELIX_RUNTIME` (here: `/home/toasty/src/helix/runtime`):

```sh
ln -sfn "$PWD/toastlang.so" "$HELIX_RUNTIME/grammars/toastlang.so"
ln -sfn "$PWD/queries"      "$HELIX_RUNTIME/queries/toastlang"
```

and in `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "toastlang"
scope = "source.tl"
file-types = ["tl"]
grammar = "toastlang"
language-servers = ["toastlang"]

[[grammar]]
name = "toastlang"
source = { path = "/home/toasty/src/ToastCPU/tree-sitter-toastlang" }
```

Confirm with `hx --health toastlang` (Tree-sitter parser ✓, Highlight queries ✓).
Restart Helix after installing — it reads grammars at startup.

## Possible additions

`queries/` currently has only `highlights.scm`. Helix can also use
`indents.scm`, `textobjects.scm`, and `injections.scm` for auto-indent, text
objects, and injected languages.
