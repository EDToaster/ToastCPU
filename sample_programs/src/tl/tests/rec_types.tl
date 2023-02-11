#include<std/io>
#include<std/assert>

struct _Outer {
    a [10] u16 
}

mod rec_types {
    using rec_types
    using assert

    struct Inner {
        a _Outer
    }

    struct InnerRec1 {
        a InnerRec1*
    }

    global inner InnerRec1 0 

    fn test -> {
        "Testing recursive types..." print
        "Inner size" { sizeof(Inner) 10 = } assert

        inner inner .a store

        "Infinite struct self-reference" { inner inner .a load .a load .a load .a load .a load .a load .a load .a load = } assert
    }
}


fn test_rec_types -> {
    rec_types::test
}