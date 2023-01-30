#include<std/io>
#include<std/string>
#include<std/mem>

#include<lib/command_buffer>

mod os {
    using os

    using io
    using command_buffer

    // GLOBALS
    global command_buffer CommandBuffer 0

    fn print_prompt -> {
        WHITE fg "> " print_string
    }

    fn initialize -> {
        keyboard_init
        command_buffer CommandBuffer::reset

        BLACK bg WHITE fg "Welcome to ToastOS!\n"   print_string
                RED   fg "32k ROM "                 print_string
                GREEN fg "16k RAM "                 print_string
                BLUE  fg "16k MEM I/O\n"            print_string
                WHITE fg
    }

    fn loop -> {
        print_prompt
        while { 1 } {
            // handle input
            while { keyboard_get_input dup } {
                dup '\n' = if {
                    print_char
                    // execute command
                    "ping" command_buffer .command 5 streq if {
                        "pong!\n" print_string
                    } else {
                        "Unknown command: " print_string 
                        command_buffer .command print_string
                        '\n' print_char
                    }

                    // reset
                    command_buffer CommandBuffer::reset
                    print_prompt
                } else {
                    // print char and push it to command buffer
                    dup print_char
                    command_buffer swap CommandBuffer::push_char
                }
            }
            drop
        }
    }
}

// register keyboard input handler
fn isr -> {
    io::keyboard_handle_input
}

fn main -> {
    os::initialize
    os::loop
}