
#include<std/io>
#include<std/mem>

struct Struct {
    a     u16
    b [2] u16
}

struct NestedStruct {
    a     u16
    inner Struct
}

global instance NestedStruct 0

fn main -> {
    instance sizeof(NestedStruct) memdump

    0xDEAD instance .a store
    0xBEEF instance .inner .a store
    0x1212 instance .inner .b store
    0x3434 instance .inner .b 1 + store

    '\n' print_char
    instance sizeof(NestedStruct) memdump
}