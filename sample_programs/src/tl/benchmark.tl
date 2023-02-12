
#include<std/io>

fn main -> {

    0 while { dup 1000 < } {
        0 while { dup 1000 < } {
            0 while { dup 1000 < } {
                ++
            } drop
            ++
        } drop
        ++

        io::print_reset "                         " io::println
        io::print_reset dup io::print_decimal " million iterations" io::println
        
    } drop
}