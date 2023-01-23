#include<std/io>
#include<std/mem>
#include<std/rand>

// register keyboard input handler
fn isr -> {
    keyboard_handle_input
}

global head_pos u16* 0

fn main -> {
    keyboard_init
    while { 1 } {
        // handle all inputs this frame
        while { keyboard_get_input dup } {
            dup 'w' = if {
                0xFF9C
            } else { dup 'a' = if {
                0xFFFF
            } else { dup 's' = if {
                100
            } else { dup 'd' = if {
                1
            } else {
                // dont move
                0
            }}}}

            // draw 0
            0x0020 head_pos load store
            head_pos load +
            // newpos
            0xFF20 over store head_pos store
            drop
        }
        // drop the 0 at the end
        drop
    }
}