#include<std/assert>
#include<std/io>

mod arith {
    using arith 

    using assert
    using io

    fn test -> {
        "Testing Arith... " print
        
        "7 = 7"     { 7 7 = }   assert
        "6 = 7"     { 6 7 = }   assert_false
        "6 > 7"     { 6 7 > }   assert_false
        "7 > 7"     { 7 7 > }   assert_false
        "8 > 7"     { 8 7 > }   assert
        "6 >= 7"    { 6 7 >= }  assert_false
        "7 >= 7"    { 7 7 >= }  assert
        "8 >= 7"    { 8 7 >= }  assert
        "6 < 7"     { 6 7 < }   assert
        "7 < 7"     { 7 7 < }   assert_false
        "8 < 7"     { 8 7 < }   assert_false
        "6 <= 7"    { 6 7 <= }  assert
        "7 <= 7"    { 7 7 <= }  assert
        "8 <= 7"    { 8 7 <= }  assert_false
    }
}


fn test_arith -> {
    arith::test
}