#include<std/assert>
#include<std/io>

mod logical {
    using logical
    using assert
    using io

    fn test -> {
        "Testing Logical... " print_string
        "1 && 1"    { 1 1 && 1 = }  assert
        "0 && 1"    { 0 1 && 0 = }  assert
        "0 || 0"    { 0 0 || 0 = }  assert
        "1 || 0"    { 1 0 || 1 = }  assert
        "1 !"       { 1   !  0 = }  assert
        "0 !"       { 0   !  1 = }  assert
    }
}

fn test_logical -> {
    logical::test
}