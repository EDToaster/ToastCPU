#include<std/io>

mod assert {
    using assert
    using io

    //        testname cond
    fn assert u16* $a -> {
        as(bool)
        if {
            //"Passed\n" ps
        } else {
            35 left_align RED fg dup print_string
            ' ' print_char
            "Assert Failed\n" print_string
            halt
        }
        drop
    }

    fn assert_false u16* $a -> {
        // todo use bool
        as(bool) ! assert
    }
}
