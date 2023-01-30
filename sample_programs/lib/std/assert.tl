#include<std/io>

// teststr, assertstr, cond
fn assert u16* u16 -> {
    if {
        //"Passed\n" ps
    } else {
        dup io::print_string
        ' ' io::print_char
        io::RED io::fg "Assert Failed\n" io::print_string
        halt
    }
    drop
}