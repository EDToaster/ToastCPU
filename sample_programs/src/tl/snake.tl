#include<std/io>
#include<std/mem>
#include<std/rand>

fn isr -> {
    keyboard_handle_input
}

global random Random 0

fn main -> {
    keyboard_init
    random 0xDEAD rand_init
    while { 1 } {
        // handle all inputs this frame
        while { keyboard_get_input dup } {
            drop

            random rand_int print_word
            ' ' print_char
        }
        drop
    }
}