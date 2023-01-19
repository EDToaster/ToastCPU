
#include<std/io>
#include<std/mem>

fn isr -> {
    keyboard_handle_input
}

fn main -> {
    keyboard_init
    while { 1 } {
        keyboard_get_input drop print_reset

        0 while { dup 60000 < } {
            1 +
        } drop
        // handle all inputs this frame
//        while { keyboard_get_input dup } {
//            print_char
//        }
//        drop
    }
}