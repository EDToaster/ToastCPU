#include<std/io>

fn isr -> {
    io::get_keyboard_ascii io::print_char
}

fn main -> {
    1 while { dup } { }
    drop
}