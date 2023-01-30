#include<std/assert>
#include<std/io>


global var1 u16 0xDEAD
fn test_global -> {
    "Testing global vars\n" io::print_string
    "var1 == 0xDEAD" { var1 load 0xDEAD = }           assert
    "var1 = 10"      { 10 var1 store var1 load 10 = } assert
}