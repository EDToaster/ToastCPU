
#include<std/io>
#include<std/string>
#include<std/mem>

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
    0x0F0F over .a store
    0x1212 over .b [0] store
    0x3434 over .b [1] store
    0x5656 over .b [2] store
    0x7878 over .b [3] store
    0x9A9A over .b [4] store
}

fn init_nested_struct NestedStruct* -> NestedStruct* {
    0xDEAD over .a store
    dup .inner ?? [0] init_struct drop
    dup .inner ?? [1] init_struct drop
    0xBEEF over .c store
}

global instance NestedStruct 0

fn main -> {
    instance sizeof(NestedStruct) memdump

    instance init_nested_struct drop

    '\n' print_char
    instance sizeof(NestedStruct) memdump
}