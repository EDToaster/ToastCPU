#include<std/io>
#include<std/rand>

using io

global color_test_r rand::Random 0
    
fn test_color -> {
    color_test_r 0xBEEF rand::rand_init

    "My Little Test Suite\n"

    while { dup load dup } {
        color_test_r rand::rand_int 7 & fg
        color_test_r rand::rand_int 7 & bg

        print_char

        1 +
    }
    drop drop
    
    WHITE fg BLACK bg
}