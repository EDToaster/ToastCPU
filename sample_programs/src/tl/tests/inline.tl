#include<std/assert>
#include<std/io>

mod inline_ {
    using inline_
    using assert
    using io

    inline INLINE_VAL   0xC0FF
    inline INLINE_BLOCK { 0xDEAD 0xBEEF + }
    inline INLINE_IF    { true if { 0xDEAD } else { 0xBEEF } }

    fn test -> {
        "Testing inline... " print

        "INLINE_VAL"    { INLINE_VAL 0xC0FF =            } assert
        "INLINE_BLOCK"  { INLINE_BLOCK 0xDE_AD 0xBE_EF + = } assert
        "INLINE_IF"     { INLINE_IF 0xDEAD =             } assert
    }
}

fn test_inline -> {
    inline_::test
}