#include<std/io>

using io

fn main -> {
    clear
    0 while { dup 8 < } {
        dup bg
        0 while { dup 8 < } {
            dup fg
            " A" print
            1 +
        } drop
        '\n' print_char
        1 +
    } drop
}