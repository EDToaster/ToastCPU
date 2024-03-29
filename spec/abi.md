- [Memory Layout](#memory-layout)
- [Register Use](#register-use)
  - [Macro Definitions](#macro-definitions)


Here lies the Application Binary Interface specification of `toast`. We will be discussion general *memory layouts* as well as *procedure calling conventions* which will outline how registers of `toast` should be used. Even though parts of this ABI may be applicable to other cpu architectures as well, it will make certain `toast`-related assumptions.

# Memory Layout

Memory consists of a 64kW addressable space.

```
R: | 32k ROM | 16k RAM | 16k I/O |
W: | 32k VGA | 16K RAM | 16k I/O |
```

Note we use the same addresses for both VGA and ROM, but this is fine because we will never be writing to ROM, and never reading from VGA. This frees up quite a bit of space for memory mapped I/O.

# Register Use

This section will outline what conditions the assembler needs to maintain when expanding a `call` macro, a `ret` macro, or a `retv` macro.

As stated in the [architecture spec](./arch.md) there are 16 16-bit registers, allocated as:

* `r0     (ar) 1 Assembler Temporary register`
* `r1-r4  (p0 to p3) 4 Function Argument Registers`
* `r5-r11 (t0 to t6) 7 General Purpose Registers (Callee must restore)`
* `r12    (isr) Interrupt service routine`
* `r13    (sp) Stack Pointer`
* `r14    (sr) Status Register`
* `r15    (pc) Program Counter`

Of which, only the `p` and `t` registers should directly be modified by the programmer. 

Note that `p` registers can be used by the caller if the callee does not expect any arguments, but they will not be restored upon return. 

Additionally, the callee may use the `p0` register to store any return values, and the caller should always use `t0` to indicate a jump location. 

## Macro Definitions

The `call rdst` macro is as it seems, pushing the return address onto the stack and jumping to the destination register.

In general (since the CPU architecture does not support a `call` instruction), a `call` macro should be expanded by the assembler as such (assuming a target location stored in `t0`):

```
// push return address onto the stack
mov      at, pc
iadd     at, 0x7    
push     at
// get jump address
mov      at, t0   
jmp      at
```

A `ret` macro assumes the value at the top of the stack is the value of the callee's return address.

```
pop     at
jmp     at
```

Note that the `call` and `ret` macros do not need to save the register, as that is the responsibility of the user `callee` function.