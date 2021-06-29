- [Memory Layout](#memory-layout)
- [Register Use](#register-use)
  - [Macro Definitions](#macro-definitions)


Here lies the Application Binary Interface specification of `toast`. We will be discussion general *memory layouts* as well as *procedure calling conventions* which will outline how registers of `toast` should be used. Even though parts of this ABI may be applicable to other cpu architectures as well, it will make certain `toast`-related assumptions.

# Memory Layout
// todo!

# Register Use

This section will outline what conditions the assembler needs to maintain when expanding a `call` macro, a `ret` macro, or a `retv` macro.

As stated in the [architecture spec](./arch.md) there are 16 16-bit registers, allocated as:

* (at) 1 Assembler Temporary register 
* (rv) 1 Return Value register
* (p0 to p2) 3 Function Argument Registers 
* (t0 to t3) 4 Temporary Scratch Registers 
* (t4 to t7) 4 Saved Temporary Registers
* (sp) Stack Pointer
* (sr) Status Register

Of which, only the `p` and `t` registers should directly be modified by the programmer. Note that p0 to p2 can be used by the caller if the callee does not expect any arguments, but they will not be restored upon return.

## Macro Definitions

The `call` macro is as it seems, preserving saved registers and jumping to another location.

In general (since the CPU architecture does not support a `call` instruction), a `call` macro should be expanded by the assembler as such (assuming a target location of 0xABCD):

```
// save registers
push     t5
push     t6
push     t7
push     t8
// get return address
mov      at, pc
iadd     at, 0x7    
push     at
// get jump address
imov     at, 0xAB
ishl     at, 8
imov     t5, 0xCD
or       at, t5
jnz      at
```

A `ret` macro assumes the value at the top of the stack is the value of the caller's `r8` register.

```
// restore registers
pop     at
pop     t8
pop     t7
pop     t6
pop     t5
mov     at, at  // this will be cleaner once I implement a non-conditional jump
jnz     at
```
