#include<std/io>

fn test_color -> {
    io::RED     io::fg 'M' io::print_char
    io::YELLOW  io::fg 'u' io::print_char
    io::GREEN   io::fg 'l' io::print_char
    io::CYAN    io::fg 't' io::print_char
    io::MAGENTA io::fg 'i' io::print_char
    io::WHITE   io::fg "-Colored!\n" io::print_string
}