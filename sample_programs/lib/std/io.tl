#include<std/mem>
#include<std/math>
#include<std/collection>
#include<std/addrs>

mod io {
    using io
    using collection

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
        swap 11 << | text_style store
    }

    fn bg u16 -> {
        0b1111100011111111 text_style load &
        swap 8 << | text_style store
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

    fn newline -> {
        text_ptr load text_progress load + text_ptr store
        100 text_progress store
    }

    // todo: optimize
    fn print_char u16 -> {
        dup '\n' = if {
            drop newline
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
        0xF & print_hex
    }

    // Print the word
    //    val
    fn print_word u16 -> {
        dup 8 >> 0xFF & print_nibble
        0xFF & print_nibble
    }

    fn print_ptr $a* -> {
        as(u16) print_word
    }

    inline _divmod_print {
        // div:u16 -> 
        math::divmod swap dup as(bool) do_print load || dup do_print store if {
            '0' + print_char
        } else {
            drop
        }
    }

    fn print_decimal u16 -> {
        false let do_print {
            10000   _divmod_print
            1000    _divmod_print
            100     _divmod_print
            10      _divmod_print
            '0' +   print_char
        }
    }

    //     strptr
    fn print u16* -> {
        while { dup load dup } {
            // str char
            print_char 1 +
        }
        drop drop
    }

    fn println u16* -> {
        print '\n' print_char
    }

    global key_buffer           RingBuffer  0
    global key_buffer_raw [32]  u16         0

    fn keyboard_init -> {
        key_buffer key_buffer_raw 5 RingBuffer::init
    }

    fn keyboard_handle_input -> {
        key_buffer get_keyboard_ascii RingBuffer::enqueue
    }

    fn keyboard_get_input -> u16 bool {
        key_buffer RingBuffer::dequeue
    }

    // inputs
    fn get_keyboard_ascii -> u16 {
        "              `      q1   zsaw2  cxde43   vftr5  nbhgy6   mju78  ,kio09  ./l;p-     [=    \n] \\                                                                                                                                                                  "
        addrs::KEYBOARD load + load
    }
}
