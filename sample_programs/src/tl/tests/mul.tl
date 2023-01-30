#include<std/assert>
#include<std/io>

using assert
using io

// multiply by repeated add
fn mul u16 u16 -> u16 {
    0 let a b acc {
        while { b load } {
            // acc += a
            acc load { a load } + acc store
            // b -= 1
            b load 1 - b store
        }
        acc load
    }
}

fn test_mul -> {
    "Testing mul\n" print_string
    "1 x 2"  { 1 2 mul 2 = }  assert
    "2 x 5"  { 2 5 mul 10 = } assert
    "0 x 10" { 0 10 mul 0 = } assert
    "10 x 0" { 10 0 mul 0 = } assert
}