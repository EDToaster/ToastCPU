#include<std/io>
#include<std/rand>
#include<std/assert>

mod color {
    using color
    using io
    using assert

    fn test -> {
        get_style

        RED     fg "Toast"      print
        YELLOW  fg "Lang "      print
        GREEN   fg "Test "      print
        BLUE    fg "Suite... "  print

        "Black style" { BLACK fg BLACK bg get_style 0 =      } assert
        "White style" { WHITE fg WHITE bg get_style 0x3F00 = } assert

        // set the style back to normal
        set_style
    }
}

    
fn test_color -> {
    color::test
}