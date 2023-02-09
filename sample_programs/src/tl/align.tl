#include<std/io>

using io

fn main -> {
    0 while { dup 100 < } {
        dup left_align ":)\n" print
        2 +
    }
    drop
}