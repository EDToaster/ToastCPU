
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

## Todo
- [ ] Add static type checking to functions
- [ ] Add structs support
- [x] Add global static memory allocation
  - `global foo 100` Denotes global variable called `foo` which will be initialized with `100`