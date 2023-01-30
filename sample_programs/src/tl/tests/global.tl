#include<std/assert>
#include<std/io>

using assert
using io

global var1 u16 0xDEAD

fn test_global -> {
    "Testing global vars\n" print_string
    "var1 == 0xDEAD" { var1 load 0xDEAD = }           assert
    "var1 = 10"      { 10 var1 store var1 load 10 = } assert
}