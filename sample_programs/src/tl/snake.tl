#include<std/io>
#include<std/mem>
#include<std/rand>

// register keyboard input handler
fn isr -> {
    io::keyboard_handle_input
}

fn main -> {
    snake::main
}

mod snake {
    using snake

    using io
    using rand

    mod dir {
        inline RIGHT    0
        inline DOWN     1
        inline LEFT     2
        inline UP       3
        inline NONE     4
    }

    global head      u16* 0
    global direction u16  4

    global style     u16  0x3F

    global random Random 0

    fn main -> {
        random 0xDEAD rand_init
        keyboard_init
        while { 1 } {
            // handle all inputs this frame
            while { keyboard_get_input } {
                dup 'w' = if {
                    dir::UP direction store
                } else { dup 'a' = if {
                    dir::LEFT direction store
                } else { dup 's' = if {
                    dir::DOWN direction store
                } else { dup 'd' = if {
                    dir::RIGHT direction store
                }}}}
                drop
            }

            // move head
            direction load 
            dup dir::UP = if {
                0xFF9C
            } else { dup dir::RIGHT = if {
                0x0001
            } else { dup dir::DOWN = if {
                0x0064
            } else { dup dir::LEFT = if {
                0xFFFF
            } else { 0 }}}}
            swap drop head load + dup head store

            style load 8 << 0x20 | swap store

            // style load 1 + 0x3F & style store

            0 while { dup 30000 < } { 1 + } drop

            // drop the 0 at the end
            drop
        }
    }
}
