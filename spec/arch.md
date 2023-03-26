- [Overview](#overview)
- [Addresses and Memory Layout](#addresses-and-memory-layout)
- [Registers](#registers)
  - [Status Register](#status-register)
- [Instructions and Opcodes](#instructions-and-opcodes)
- [Todo](#todo)

# Overview

The `toast` CPU architecture is:

* 16-bit
* Register Based
* a von Neumann Model
* Multi-Cycle anti-MIPs
* Minimal

The `toast` CPU architecutre is *not*:
 
* Fast
* Efficient
* Practical


# Addresses and Memory Layout

All addresses and datasizes are 16 bits wide. This means that `toast` has a total of 2^16 = 64 kilowords (kw) of addressable memory, at a total of 1 Mbits. The address space is split up into 3 sections:

1. 32 kw of addressable ROM
2. 16 kw of General Purpose RAM
3. 16 kw of General Memory Mapped IO

# Registers

(Read more about registers and calling conventions in the [application binary interface](abi.md) document)

There are a total of 16 16-bit integer registers:

* `r0     (ar) 1 Assembler Temporary register`
* `r1-r4  (p0 to p3) 4 Function Argument Registers`
* `r5     (v0) Return value` 
* `r6-r11 (t0 to t5) 7 General Purpose Registers (Callee must restore)`
* `r12    (isr) Interrupt service routine`
* `r13    (sp) Stack Pointer (0xC000 downwards)`
* `r14    (sr) Status Register`
   * WIP: Bits 15-8: Register bank #
   * Bit 4: V, if last operation caused overflow
   * Bit 3: C, if last operation caused carry
   * Bit 2: N, if last operation produced a negative number
   * Bit 1: Z, if last operation produced 0x0000
   * Bit 0: X, if last operation produced 0xFFFF
* `r15    (pc) Program Counter`

## Status Register

// todo: table of which status bits are read from/written to 

# Instructions and Opcodes

Each instruction of `toast` is 16 bits wide, prepended by a 4-bit opcode.

There are 5 types of instructions (x denotes set bit, - denotes "I don't care what the value is")

| Type | Layout                                                 |
|:-----|:-------------------------------------------------------|
| D    | <pre>xxxx xxxx xxxx ----<br>op   rx1  rx2       </pre> |
| I    | <pre>xxxx xxxx xxxxxxxx <br>op   rx1  imm8      </pre> |
| J    | <pre>xxxx xxxx -- xx xxxx<br>op   rdst    rl jop</pre> |
| ALU  | <pre>xxxx xxxx xxxx xxxx<br>op   rx1  rx2  aluop</pre> |
| HALT | <pre>0111 ---- ---- ----<br>halt                </pre> |
| RTI  | <pre>1100 ---- ---- ----<br>rti                 </pre> |

However, for the negate operation (ALU-type), `rx2` is always 0. 
For iALU ops, rx2 is an imm4 value.

J-type operations are used for jump operations, where `n` negates the jump condition.

```
0000 xxxx xxxx xxxx
load rx1  rx2  offset
     (D-type) Move the data pointed to by rx2 into the register rx1 (with offset)

0001 xxxx xxxx xxxx
str  rx1  rx2  offset
     (D-type) Move the data in register rx2 into memory location rx1 (with offset)

0010 xxxx xxxxxxxx
imov rxx  imm8
     (I-type) Move imm8 (sign extended to 16 bits) into register rxx

0011 xxxx xxxxxxxx
imoh rxx  imm8
     (I-type) Move imm8 into high byte of register rxx

0100 Unused

0101 xxxx xxxx ----
push rxx  rsrc
     (D-type) Write data in rsrc into mem addr at rxx, then decrement rxx.

0110 xxxx xxxx ----
pop  rdst rxx
     (D-type) Increment rxx, then read the mem addr at rxx, storing in rdst.

0111 ---- ---- ----
halt
     (HALT-type) Halt the CPU

1000 xxxx xxxx xxxx
(op) rx1  rx2  aluop
     (ALU-type) rx1 := rx1 (op) rx2
                op   = aluop
                not  = 0000
                and  = 0001
                or   = 0010
                xor  = 0011
                add  = 0100
                sub  = 0101
                mov  = 0110
                cmp  = 0111
                shr  = 1000
                sshr = 1001
                shl  = 1010

                cmp will emulate a subtraction, and set all 
                of the flags, but will not write the result 
                back into the register.

1001 xxxx xxxx xxxx
(op) rx1  imm4 aluop
     (iALU-type) rx1 := rx1 (op) imm4
                 imm4 is 0-extended to 16-bit value
                 iop   = aluop
                 inot  = 0000 // equivalent to not
                 iand  = 0001
                 ior   = 0010
                 ixor  = 0011
                 iadd  = 0100
                 isub  = 0101
                 imov  = 0110  // actually useless
                 icmp  = 0111
                 ishr  = 1000
                 isshr = 1001
                 ishl  = 1010

                 `tst rx` is an alias for:
                    icmp rx 0

1010 xxxx --xx xxxx
(op) rdst   rl jop
     (J-type) Jump to the destination described by the value of rdst using the following conditions 
              jop : condition
              0000: (jmp) unconditional jump
              0001: (jz)  jump if zero
              0010: (jnz) jump if not zero
              0011: (jn)  jump if negative
              0100: (jp)  jump if positive

              rl: (00, 01, 10) 

              l : if the l bit is set and a jump is performed, push 
                  the next instruction's pointer onto the stack.

              r : if the r bit is set and we will perform a jump, pop
                  the stack and jump to that popped value. When 
                  the r bit is set, rdst is completely ignored.

1011 (unused)

1100 ---- ---- ----
rti
     (RTI-type) Return from interrupt
                Pops the top of stack into status register, then pops the top
                of stack into program counter, executing a jump. `rti` will also
                signal to the CPU to start accepting interrupts again.
```

# Todo
- [x] Change all J-type instructions so that the `opcode` is the same, and use last four bits to differentiate between `jz`, `jeq`, etc
- [x] Implement hardware interrupts
- [ ] Implement `rti`
- [x] Add load with offset instruction and move stack to 0xBFFF