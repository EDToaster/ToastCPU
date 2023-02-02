#include<std/mem>

mod io {
    using io

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

    fn clear -> {
        print_reset
        0 while { dup 6000 < } {
            ' ' print_char
            1 + 
        } drop
        print_reset
    }

    fn fg u16 -> {
        0b1100011111111111 text_style load &
        over 11 << | text_style store drop
    }

    fn bg u16 -> {
        0b1111100011111111 text_style load &
        over 8 << | text_style store drop
    }

    fn get_style -> u16 {
        text_style load
    }

    fn set_style u16 -> {
        text_style store
    }

    fn print_reset -> {
        100 text_progress store
        0x0000 as(u16*) text_ptr store
    }

    // left align to column
    // left_align(0) brings next print to the start of the line
    fn left_align u16 -> {
        // new_text_progress = 100 - x
        100 swap -

        // offset = text_progress - new_text_progress
        dup text_progress load swap -

        // new_tp offset

        // store new_tp to text_progress
        over text_progress store

        // add offset to text_ptr
        text_ptr load + text_ptr store

        // drop new_tp
        drop
    }

    // todo: optimize
    fn print_char u16 -> {
        dup '\n' = if {
            drop
            text_ptr load text_progress load + text_ptr store
            100 text_progress store
        } else {
            text_ptr load swap

            // print the char
            text_style load |
            over store

            1 + text_ptr store

            text_progress load 1 - dup 0 = if {
                drop 100
            }
            text_progress store
        }

        text_ptr load as(u16) 6000 >= if {
            print_reset
        }
    }

    // Print the hex char
    fn print_hex u16 -> {
        "0123456789ABCDEF" + load print_char
    }

    // Print the nibble
    //     val
    fn print_nibble u16 -> {
        dup 4 >> 0xF & print_hex
        dup 0xF & print_hex
        drop
    }

    // Print the word
    //    val
    fn print_word u16 -> {
        dup 8 >> 0xFF & print_nibble
        dup 0xFF & print_nibble
        drop
    }

    fn print_ptr $a* -> {
        as(u16) print_word
    }

    //     strptr
    fn print_string u16* -> {
        dup
        while { load dup } {
            // str char
            print_char 1 + dup
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

    global key_buffer KeyBuffer 0

    fn keyboard_init -> {
        key_buffer as(u16*) 0 sizeof(KeyBuffer) memset
    }

    fn keyboard_handle_input -> {
        // write the current position
        get_keyboard_ascii
        key_buffer .buffer key_buffer .write_offset load +
        store

        // update position
        key_buffer .write_offset load 1 + 0x1F & key_buffer .write_offset store
    }

    fn keyboard_get_input -> u16 {
        // while r != w
        key_buffer .read_offset load key_buffer .write_offset load = ! if {
            // read current char
            key_buffer .buffer key_buffer .read_offset load + load

            // advance read_offset
            key_buffer .read_offset load 1 + 0x1F & key_buffer .read_offset store
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
