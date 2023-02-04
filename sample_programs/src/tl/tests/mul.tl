#include<std/assert>
#include<std/io>
#include<std/math>

mod mul {
    using mul
    using assert
    using io
    using math

    fn test -> {
        "Testing mul... " print_string
        "1 x 2"  { 1 2 mul 2 = }  assert
        "2 x 5"  { 2 5 mul 10 = } assert
        "0 x 10" { 0 10 mul 0 = } assert
        "10 x 0" { 10 0 mul 0 = } assert
    }
}

fn test_mul -> {
    mul::test
}