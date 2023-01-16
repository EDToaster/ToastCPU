
#include<std/io>
#include<std/mem>

global arr[16] u16 0

fn print_arr -> {
    0 while { dup 16 < } {
        arr over + load print_word
        1 +
    }
    drop
}

fn main -> {
    print_arr
    arr 0xDEAD 16 memset
    print_arr
}