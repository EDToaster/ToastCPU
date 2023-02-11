
#include<std/io>

fn main -> {

    0 while { dup 1000 < } {
        0 while { dup 1000 < } {
            0 while { dup 1000 < } {
                1 +
            } drop
            1 +
        } drop
        1 +

        io::clear
        dup io::print_decimal " million iterations" io::println
        io::print_reset
    } drop
}