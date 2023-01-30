#include<std/mem>

mod io {
    global text_progress u16    100
    global text_ptr      u16*   0x0000

    global text_style u16 0b0011100000000000

    inline BLACK    0b000
    inline BLUE     0b001
    inline GREEN    0b010
    inline CYAN     0b011
    inline RED      0b100
    inline MAGENTA  0b101
    inline YELLOW   0b110
    inline WHITE    0b111

    fn fg u16 -> {
        0b1100011111111111 io::text_style load &
        over 11 << | io::text_style store drop
    }

    fn bg u16 -> {
        0b1111100011111111 io::text_style load &
        over 8 << | io::text_style store drop
    }

    fn print_reset -> {
        100 io::text_progress store
        0x0000 as(u16*) io::text_ptr store
    }

    fn print_char u16 -> {
        dup '\n' = if {
            drop
            io::text_ptr load io::text_progress load + io::text_ptr store
            100 io::text_progress store
        } else {
            io::text_ptr load swap

            // print the char
            io::text_style load |
            over store

            1 + io::text_ptr store

            io::text_progress load 1 - dup 0 = if {
                drop 100
            }
            io::text_progress store
        }

        io::text_ptr load as(u16) 6000 >= if {
            0 as(u16*) io::text_ptr store
            100 io::text_progress store
        }
    }

    // Print the hex char
    fn print_hex u16 -> {
        dup 10 < if {
            '0' + io::print_char
        } else {
            10 - 'A' + io::print_char
        }
    }

    // Print the nibble
    //     val
    fn print_nibble u16 -> {
        dup 4 >> 0xF & io::print_hex
        dup 0xF & io::print_hex
        drop
    }

    // Print the word
    //    val
    fn print_word u16 -> {
        dup 8 >> 0xFF & io::print_nibble
        dup 0xFF & io::print_nibble
        drop
    }

    //     strptr
    fn print_string u16* -> {
        dup
        while { load dup } {
            // str char
            io::print_char 1 + dup
        }
        drop drop
    }

    // 1 2 3 4 5 6 ...
    // r   w
    // while r != w, read
    struct KeyBuffer {
        // the next position to read
        read_offset  u16
        // the next position to write
        write_offset u16
        buffer [32]  u16
    }

    global key_buffer io::KeyBuffer 0

    fn keyboard_init -> {
        io::key_buffer as(u16*) 0 sizeof(io::KeyBuffer) memset
    }

    fn keyboard_handle_input -> {
        // write the current position
        io::get_keyboard_ascii
        io::key_buffer .buffer io::key_buffer .write_offset load +
        store

        // update position
        io::key_buffer .write_offset load 1 + 0x1F & io::key_buffer .write_offset store
    }

    fn keyboard_get_input -> u16 {
        // while r != w
        io::key_buffer .read_offset load io::key_buffer .write_offset load = ! if {
            // read current char
            io::key_buffer .buffer io::key_buffer .read_offset load + load

            // advance read_offset
            io::key_buffer .read_offset load 1 + 0x1F & io::key_buffer .read_offset store
        } else {
            0
        }
    }

    // inputs
    fn get_keyboard_ascii -> u16 {
        "              `      q1   zsaw2  cxde43   vftr5  nbhgy6   mju78  ,kio09  ./l;p-     [=    \n] \\                                                                                                                                                                  "
        0xFFFF as(u16*) load + load
    }
}
