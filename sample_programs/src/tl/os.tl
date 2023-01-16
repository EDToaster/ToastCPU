
#include<std/io>

struct KeyBuffer {
    keystroke_read_offset    u16
    keystroke_buffer[16]     u16
}

// todo: implicitly define these new symbols
// sizeof(KeyBuffer)
// .keystroke_read_offset KeyBuffer* -> u16*
// .keystroke_buffer      KeyBuffer* -> u16*

// todo: rework syntax
// allocate some buffer for incoming keystrokes
global keybuffer KeyBuffer 0

fn isr -> {
    // todo: use interrupt codes
    //keystroke_offset load 1 + 0x0F &
    //get_keyboard_ascii over keystroke_buffer + store keystroke_offset store
}

fn main -> {
    sizeof(u16) print_word
}