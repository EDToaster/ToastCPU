#include<std/assert>
#include<std/io>

mod logical {
    using logical
    using assert
    using io

    fn test -> {
        "Testing Logical... " print_string
        "1 && 1"    { true true && }    assert
        "0 && 1"    { false true && }   assert_false
        "0 || 0"    { false false || }  assert_false
        "1 || 0"    { true false || }   assert
        "1 !"       { true   ! }        assert_false
        "0 !"       { false  ! }        assert
    }
}

fn test_logical -> {
    logical::test
}