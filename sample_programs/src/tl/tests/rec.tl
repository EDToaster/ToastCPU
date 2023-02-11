#include<std/assert>
#include<std/io>

mod rec {
    using rec
    using assert
    using io

    fn fib u16 -> u16 {
        dup 1 >  if {
            dup 1 - fib
            swap 2 - fib
            +
        }
    }

    fn test -> {
        "Testing Recursion... " print
        
        "fib(7)"  { 7 fib 13 =    } assert
        "fib(18)" { 18 fib 2584 = } assert
    }
}

fn test_rec -> {
    rec::test
}