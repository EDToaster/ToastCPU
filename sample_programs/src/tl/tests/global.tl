#include<std/assert>
#include<std/io>

mod global_ {
    using global_

    using assert
    using io

    global var1 u16 0xDEAD

    fn test -> {
        "Testing global vars... " print_string
        "var1 == 0xDEAD" { var1 load 0xDEAD = }           assert
        "var1 = 10"      { 10 var1 store var1 load 10 = } assert
    }
}

fn test_global -> {
    global_::test
}