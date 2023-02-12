
#include<std/io>
#include<std/assert>

mod func_ptr {
    using func_ptr
    using io
    using assert

    fn add_1 u16 -> u16 {
        ++
    }

    fn mul_2 u16 -> u16 {
        1 <<
    }
    
    fn do_twice $a ($a -> $a)* -> $a {
        swap over () swap () 
    }

    // keeps applying function until greater than 100
    fn do_many u16 (u16 -> u16)* -> u16 {
        while { over 100 < } {
            swap over () swap
        }
        drop
    }

    // just testing that this is compilable
    fn do_growing (u16 -> u16 u16)* -> {
        while { false } {
            1 over () drop drop
        }
        drop
    }
    
    // this doesn't work for now.
    // fn generic_hackery $a (u16 -> $a)* -> $a $a {
    //     1 swap ()
    // }
    
    fn mismatched_generics $a (u16 -> $a)* -> {
        drop drop
    }

    fn foo u16 -> u16* {
        as(u16*)
    }

    fn test -> {
        "Testing function pointers... " print
        
        "do_twice(2, &add_1)" { 2 ptr(add_1) do_twice  4 = } assert
        "do_many(5, &mul_2)"  { 5 ptr(mul_2) do_many 160 = } assert
        "do_many(2, &mul_2)"  { 2 ptr(mul_2) do_many 128 = } assert
        
        // should not compile
        // 1 ptr(foo) mismatched_generics
    }
}

fn test_func_ptr -> {
    func_ptr::test
}
