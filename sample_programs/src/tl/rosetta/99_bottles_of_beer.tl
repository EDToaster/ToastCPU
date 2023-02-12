
#include<std/io>

using io

inline bottles {
    dup print_decimal
    1 = if {
        " bottle of beer"
    } else {
        " bottles of beer" 
    } print
}

fn main -> {
    5 while { dup 0 > } {
        dup bottles " on the wall" println
        dup bottles "" println
        "Take one down, pass it around" println
        --
        dup bottles " on the wall" println
        "" println
    }
    drop
}