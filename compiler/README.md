
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