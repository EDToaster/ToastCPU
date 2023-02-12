#include<std/io>

using io

fn main -> {
    // Use ?? (hole) operator to inspect the 
    // current state of the stack during compilation

    // The ?? operator is useful to inspect stack type-check 
    // errors during compilation, or to inspect the stack state
    // inside loops and branching statements.

    //            [u16, u16] [u16]         []
    //            v          v             v
    0xDEAD 0xBEEF ?? +       ?? print_word ??

    //     [u16*] [u16*, u16] [u16*]
    //     v      v           v
    "ABCD" ?? 1   ?? +        ?? print

    10 while { dup 0 >= } {
        // [u16]
        // v
           ?? 
        dup print_word
        --
    }
    
    // [u16]
    // v
       ??
    drop
}