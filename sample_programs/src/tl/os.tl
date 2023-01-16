
#include<std/io>

// allocate some buffer for incoming keystrokes
global keystroke_read_offset    u16    0
global keystroke_offset         u16    0
global keystroke_buffer[16]     u16    0

fn isr -> {
    // todo: use interrupt codes
    keystroke_offset load 1 + 0x0F &
    get_keyboard_ascii over keystroke_buffer + store keystroke_offset store
}

fn main -> {
    1 while { dup } {

        // todo: do printloop

    }
    drop
}