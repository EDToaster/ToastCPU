#include<std/io>
#include<std/assert>

mod bool_ {
    using bool_
    using io
    using assert

    fn take_bool bool -> { drop }

    fn ret_true -> bool { true }
    fn ret_false -> bool { false }

    fn test -> {
        "Testing bools..." print_string

        "Return true"   { ret_true  } assert 
        "Return false"  { ret_false } assert_false
        "Assert true"   { true      } assert
        "Assert false"  { false     } assert_false
    }
}


fn test_bool -> {
    bool_::test
}

