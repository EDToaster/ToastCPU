#include<std/io>

// teststr, assertstr, cond
fn assert u16* u16 -> {
    if {
        //"Passed\n" ps
    } else {
        dup print_string
        ' ' print_char
        RED fg "Assert Failed\n" print_string
        halt
    }
    drop
}