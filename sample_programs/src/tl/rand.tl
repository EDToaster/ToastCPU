#include<std/io>
#include<std/rand>

global rand Random 0

fn p u16 -> u16 {
    dup 1 & if {
            0x3F20 print_char
    } else {
        0x0020 print_char
    }
} 

fn main -> {
    rand 0xDEAD rand_init

    while { 1 } {
        rand rand_int
        unroll 16 {
            p 1 >>
        }
        drop
    }
}