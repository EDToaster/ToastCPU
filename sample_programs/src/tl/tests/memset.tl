#include<std/io>
#include<std/assert>
#include<std/string>
#include<std/mem>

mod memset {
    using memset
    using assert
    using io
    using str
    using mem

    global memset_arr[8] u16 0

    fn test -> {
        "Testing memset... " print_string
        "arr before init" { "\0\0\0\0\0\0\0\0" memset_arr 8 streq } assert
        memset_arr 0x0020 8 memset
        "arr after init" { "        " memset_arr 8 streq } assert
    }
}

fn test_memset -> {
    memset::test
}