#include<std/io>

using io

fn main -> {
    0 while { dup 8 < } {
        dup bg
        0 while { dup 8 < } {
            dup fg
            " A" print_string
            1 +
        } drop
        '\n' print_char
        1 +
    } drop
}