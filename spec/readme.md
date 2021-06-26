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

There are a total of 16 16-bit registers:

1. 13 general use registers (r0 to r12), 
2. Stack Pointer (SP, initialized to 0x8000 on reset)
3. Status Register (SR, containing processor flags):
   * Bit 4: V, if last operation caused overflow
   * Bit 3: C, if last operation caused carry
   * Bit 2: N, if last operation produced a negative number
   * Bit 1: Z, if last operation produced 0x0000
   * Bit 0: X, if last operation produced 0xFFFF

## Status Register

// todo: table of which status bits are read from/written to 

# Instructions and Opcodes

Each instruction of `toast` is 16 bits wide, prepended by a 4-bit opcode.

There are 5 types of instructions (x denotes set bit, - denotes "I don't care what the value is")

| Type | Layout                                                 |
| :--- | :----------------------------------------------------- |
| D    | <pre>xxxx xxxx xxxx ----<br>op   rx1  rx2       </pre> |
| I    | <pre>xxxx xxxx xxxxxxxx <br>op   rx1  imm8      </pre> |
| J    | <pre>xxxx xxxx x -------<br>op   rdst n         </pre> |
| ALU  | <pre>xxxx xxxx xxxx xxxx<br>op   rx1  rx2  aluop</pre> |
| HALT | <pre>0111 ---- ---- ----<br>halt                </pre> |

However, for the negate operation (ALU-type), `rx2` is always 0. 
For iALU ops, rx2 is an imm4 value.

J-type operations are used for jump operations, where `n` negates the jump condition.

```
0000 xxxx xxxx ----
load rx1  rx2
     (D-type) Move the data pointed to by rx2 into the register rx1

0001 xxxx xxxx ----
str  rx1  rx2
     (D-type) Move the data in register rx2 into memory location rx1

0010 xxxx xxxxxxxx
imov rxx  imm8
     (I-type) Move imm8 (sign extended to 16 bits) into register rxx

0011 (Unused)

0100 xxxx x --- ----
jz   rdst n
     (J-type) Jump to rdst if the previous operation resulted in a set `Z` flag. 
              If `n` is set, negate this condition.

0101 1101 xxxx ----
push      rsrc
     (D-type) Push rsrc onto the stack, then increment SP.

0110 xxxx 1101 ----
pop  rdst
     (D-type) Decrement SP, then pop from the stack, storing in rdst.

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
                shr  = 1000
                sshr = 1001
                shl  = 1010

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
                 ishr  = 1000
                 isshr = 1001
                 ishl  = 1010
```

# Todo
- [ ] Change all J-type instructions so that the `opcode` is the same, and use last four bits to differentiate between `jz`, `jeq`, etc
- [ ] Implement hardware interrupts