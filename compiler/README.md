# ToastLang

ToastLang (`tl`) is a compiled language that compiles down to `tasm` (see: `assembler`) which can be further compiled 
down to the `Toast architecture instruction set`. The resulting binary (`.mif` file) can be
run through the emulated ToastCPU or fpga implementation.

`tl` is a stack-based concatenative language, similar to `Forth`, `Porth`, and `Joy`. However,
unlike a pure stack-based language, it supports static allocation on the heap, and uses an
additional `return` stack to retain information about function call return address and any
local variable bindings. Think of this additional `return` stack as a more traditional stack-frame 
calling architecture.

It also supports global static memory allocations with name bindings, unlike `Forth`.

## Features

The following list of features are subject to change as development continues.

### Hello World
```toastlang
fn main -> {
    "Hello from a piece of toast!\n" ps
}
```

### Basic Arithmetic
```toastlang
fn main -> {
  1 2 + p
}
```

This piece of code pushes `1` and `2` on to the stack. Then, the `+` operator pops two values from the stack and pushes 
the sum on to the stack. The `p` function prints out the resulting number in hex notation.

### Turing Completeness

Very technically (and not *really* rooted in reality), `toastlang` (and by extension, `toastasm` and `toastcpu`) is turing complete, since it is able to implement a 100-cell wide 
r110 cellular automata.

![Rule 110 Implemented on tl](./assets/r110.png)

## Todo
- [x] Add static type checking to functions
- [x] File include system
- [ ] Deprecate file include system and add actual modules
- [x] Add structs support
- [x] Add const array offset support `0xDEAD ptr [2] store`
- [ ] Add better control flow
  - [x] `return` Jump to end of function 
    - [ ] todo: Make better control flow checks when returning or breaking 
  - [ ] `break` Jump out of current loop
  - [ ] Refactor type checking to account for early return or break
  - [ ] Implement switch/match statements + if/elseif/else statements, and lower them to nested ifs.
- [ ] Better static compiler check error messages
- [x] Add global static memory allocation
  - [x] `global foo u16 100` Denotes global variable called `foo` which will be initialized with `u16: 100`
  - [x] `global foo [12] u16 0` Add support for array allocation in global variables
- [ ] Add `const` and `inline`
  - [ ] `const foo u16 100` pushes the address of foo to the stack. Similar to strings but better and reused!
  - [x] `inline foo 100` pushes the *value* of foo to the stack. The substitution happens at compile time, and 
        doesn't require any rom allocation. Inline values are expanded out like macros.
- [x] Add support for pattern matched type defs
  - `pub fn foo $a $a* $b -> $b* { ... }` Generics!
- [ ] Dead code elimination
  - [x] Function tree-shake / dead-code elimination
  - [ ] Global allocation
- [ ] Module system to prevent nameclash: `mod io { struct a ... fn b ... }` referenced as `io::a` and `io::b`
  - [x] Basic functionality
  - [x] Add `using` keyword to remove the need for prepending `io::`, for example.
  - [x] See `Rough edges around modules`
    - [ ] Clean up the code though...
  - [ ] Instead of searching through all `using`s, create a datastructure to map shortnames -> items and pass that around
- [ ] Function pointers
  - [x] Express types like `(u16 -> u16)` or `($a -> u16*)`
  - [ ] Fix generic function type annotation: see Appendix 1 
  - [x] Better parsing of types instead of overloading identifiers
  - [ ] Better parsing of module member types instead of overloading identifier
- [ ] Recursive struct definitions if size is known at compile time

## Known issues

### Rough edges around modules

Currently, modules are a name-mangling mess. In addition to recursive struct definitions, we need a better way to access types inside of modules, while outside the module.

For example, this snippet will compile with an error like `Type Foo not in scope` for the `global`, `struct`, and `fn` definitions.

```rust
mod module {
    struct Foo { }
}

// implicit root module
global f Foo 0

struct Bar { 
    a Foo
}

fn func Foo* -> Foo* { ... }
```

There is no reason why `Foo` cannot be accessed outside of the submodule `module`. 

The `global` and `fn` issues are easy to fix - just process all `struct` definitions before processing any of the `global` or `fn` definitions.

However, `struct` members containing types within submodules are harder to fix. This is because we have to settle on an order to process `struct` defs -- do we process this module before processing submodules? Or do we process submodules first?

One possible solution to fix this issue is to gather all struct name declarations first (in all places), *then* we will try to populate the sizes of each of the struct declarations, at the same time trying to populate the sizes of each type found in its members. At this time, we can calculate offsets of each of the members of the struct.

If the query for the size of its member (and their members) involves querying the size of itself, then the size of itself is undecidable and we *have to* throw `Recursive struct definitions without indirection are not allowed`

## Appendix

### Problem 1

This doesn't work

```
fn generic_hackery $a (u16 -> $a)* -> {
  1 () drop drop
}
```

It results in `Output generic "a" has no corresponding input type.`

Why? 

Because when we call the generic function using the `()` operator, we try to resolve the type `$a` within `generic_hackery`. This is not ideal as we only need to make sure that the two `$a` are the same. This can (and is) done when `generic_hackery` is called from other functions. 

I don't know how to fix this and I don't really know if it matters...