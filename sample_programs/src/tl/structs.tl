
#include<std/io>
#include<std/mem>

struct Struct {
    a     u16
    b [2] u16
}


struct NestedStruct {
    a         u16
    inner [2] Struct
}

fn init_struct Struct* -> Struct* {
    0xBEEF over .a store
    0x1212 over .b [0] store
    0x3434 over .b [1] store
}

fn init_nested_struct NestedStruct* -> NestedStruct* {
    0xDEAD over .a store
    dup .inner ?? [0] init_struct drop
    dup .inner ?? [1] init_struct drop
}

global instance NestedStruct 0

fn main -> {
    instance sizeof(NestedStruct) memdump

    instance init_nested_struct drop

    '\n' print_char
    instance sizeof(NestedStruct) memdump
}