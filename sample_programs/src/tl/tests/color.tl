#include<std/io>
#include<std/rand>
#include<std/assert>

mod color {
    using color
    using io
    using assert

    fn test -> {
        get_style

        RED     fg "My "        print_string
        YELLOW  fg "Little "    print_string
        GREEN   fg "Test "      print_string
        BLUE    fg "Suite... "  print_string

        BLACK fg BLACK bg "Black style" { get_style 0 = } assert
        WHITE fg WHITE bg "Black style" { get_style 0x3F00 = } assert

        // set the style back to normal
        set_style
    }
}

    
fn test_color -> {
    color::test
}