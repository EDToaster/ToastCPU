#include<std/assert>
#include<std/io>

mod inline_ {
    using inline_
    using assert
    using io

    inline INLINE_VAL 0xC0FF
    inline INLINE_BLOCK { 0xDEAD 0xBEEF + }

    fn test -> {
        "Testing inline... " print_string

        "INLINE_VAL"    { INLINE_VAL 0xC0FF = } assert
        "INLINE_BLOCK"  { INLINE_BLOCK 0xDEAD 0xBEEF + = } assert
    }
}

fn test_inline -> {
    inline_::test
}