#include<std/io>
#include<std/assert>
#include<std/string>
#include<std/mem>

using assert
using io

global memset_arr[8] u16 0

fn test_memset -> {
    "Testing memset\n" print_string
    "arr before init" { "\0\0\0\0\0\0\0\0" memset_arr 8 streq } assert
    memset_arr 0x0020 8 memset
    "arr after init" { "        " memset_arr 8 streq } assert
}