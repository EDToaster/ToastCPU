Following https://sourcecodeartisan.com/2020/09/13/llvm-backend-1.html

16-bit data width
16-bit address bus width

≡≡≡ Memory ≡≡≡

Total of 64 kw addressable RAM

0xxx xxxx xxxx xxxx 32kw addressable ROM
10xx xxxx xxxx xxxx 16kw addressable General Purpose RAM 
                    Stack Pointer is initialized to 0x8000
                    upon boot or reset.
11xx xxxx xxxx xxxx 16kw addressable General IO mapped memory

≡≡≡ Registers ≡≡≡

16 registers are 16 bits each

Only registers 0-12 are able to be the destination of MOV and ALU.

R0 - R7: Bank of 8 Registers  [A, B, C, D, E, F, G, H] # will be restored
R8 - R12: General Purpose     [I, J, K, L, M]          # scratch pad   
R13: Stack Pointer
R14: Status Register
     [15:8] Register Bank Select
     [7:0]  (. . . V C N Z X)
            V: 1 if last operation caused overflow
            C: 1 if last operation caused carry
            N: 1 if last result was negative
            Z: 1 if last result was 0x0000
            X: 1 if last result was 0xFFFF
R15: Program Counter

≡≡≡ Instructions ≡≡≡

18 basic 16-bit instructions

The Instruction Opcode is 4 bits

# Data movement

0000 xxxx xxxx ----
load rx1, [rx2]
     Move the data pointed to by rx2 into rx1

0001 xxxx xxxx ----
str  [rx1], rx2
     Move rx2 into the data pointed by rx1
     

0010 xxxx xxxxxxxx
imov rxx, imm8
     Move imm8 into rxx, sign extend to 16 bits

# Jumps/Halts
0100 xxxx x --- ----
jz   rdst n
     Jump to rdst if the previous operation did result in zero, negated by n

# push pop
0101 1101 xxxx ----
push      rsrc
     Push rsrc onto the stack, incrementing the stack pointer

0110 xxxx 1101 ----
pop  rdst
     Pop rdst from the stack, decrementing the stack pointer

0111
halt
     Halt the CPU

# Arithmetic operations

1000 xxxx ---- 0000
not  rx1
     rx1 := ~rx1

1000 xxxx xxxx 0001
and  rx1, rx2
     rx1 := rx1 & rx2

1000 xxxx xxxx 0010
or   rx1, rx2
     rx1 := rx1 | rx2

1000 xxxx xxxx 0011
xor  rx1, rx2
     rx1 := rx1 ^ rx2

1000 xxxx xxxx 0100
add  rx1, rx2
     rx1 := rx1 + rx2

1000 xxxx xxxx 0101
sub  rx1, rx2
     rx1 := rx1 - rx2

1000 xxxx xxxx 0110
mov  rx1, rx2
     rx1 := rx2

1000 xxxx xxxx 1000
shr  rx1  rx2
     rx1 := rx1 >> rx2

1000 xxxx xxxx 1001
sshr rx1  rx2
     rx1 := rx1 >>> rx2

1000 xxxx xxxx 1010
shl  rx1  rx2
     rx1 := rx1 << rx2

# Immediate versions of Arithmetic operations 
# imm4 is unsign-extended to 16 bits
1001 xxxx xxxx xxxx
i(op)rxx  imm4 opcode
     rxx := rxx (op) imm4

