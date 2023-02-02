#include<std/assert>
#include<std/io>

mod arith {
    using arith 

    using assert
    using io

    fn test -> {
        "Testing Arith... " print_string
        "7 = 7"     { 7 7 = 1 = }   assert
        "6 = 7"     { 6 7 = 0 = }   assert
        "6 > 7"     { 6 7 > 0 = }   assert
        "7 > 7"     { 7 7 > 0 = }   assert
        "8 > 7"     { 8 7 > 1 = }   assert
        "6 >= 7"    { 6 7 >= 0 = }  assert
        "7 >= 7"    { 7 7 >= 1 = }  assert
        "8 >= 7"    { 8 7 >= 1 = }  assert
        "6 < 7"     { 6 7 < 1 = }   assert
        "7 < 7"     { 7 7 < 0 = }   assert
        "8 < 7"     { 8 7 < 0 = }   assert
        "6 <= 7"    { 6 7 <= 1 = }  assert
        "7 <= 7"    { 7 7 <= 1 = }  assert
        "8 <= 7"    { 8 7 <= 0 = }  assert
    }
}


fn test_arith -> {
    arith::test
}