#include<std/string>
#include<std/assert>
#include<std/io>

using assert
using io

// todo: make better syntax :(
struct Struct {
    a     u16
    b [5] u16
}

struct NestedStruct {
    a         u16
    inner [2] Struct
    c         u16
}

fn init_struct Struct* -> Struct* {
    0x0041 over .a store
    0x0042 over .b [0] store
    0x0043 over .b [1] store
    0x0044 over .b [2] store
    0x0045 over .b [3] store
    0x0046 over .b [4] store
}

fn init_nested_struct NestedStruct* -> NestedStruct* {
    0x005A over .a store
    dup .inner [0] init_struct drop
    dup .inner [1] init_struct drop
    0x005A over .c store
}

global struct_instance NestedStruct 0

fn test_struct -> {
    "Testing struct... " print_string

    "sizeof(Struct)"        { sizeof(Struct) 6 =        } assert
    "sizeof(NestedStruct)"  { sizeof(NestedStruct) 14 = } assert

    struct_instance init_nested_struct drop
    "init_nested_struct" { "ZABCDEFABCDEFZ" struct_instance as(u16*) sizeof(NestedStruct) streq } assert
}
