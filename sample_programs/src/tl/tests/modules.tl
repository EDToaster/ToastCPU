
#include<std/assert>
#include<std/io>

mod module {
    inline INLINE_VAL 0xDEAD
    inline INLINE_BLOCK { 1 2 + }

    struct Struct {
        a u16
    }

    // requires fully qualified name for now
    fn init_struct module::Struct* -> module::Struct* {
        0xBEEF over .a store
    }

    global g module::Struct 0

    fn foo -> u16 {
        0xBEEF
    }

    mod nested {
        // should not collide!
        fn foo -> u16 {
            0xABAB
        }
    }
}

fn test_modules -> {
    "Testing Modules\n" io::print_string
    
    "Inline val" { module::INLINE_VAL 0xDEAD = } assert
    "Inline block" { module::INLINE_BLOCK 3 = } assert

    module::g module::init_struct drop

    "module::foo" { module::foo 0xBEEF = } assert
    "module::g .a load" { module::g .a load 0xBEEF = } assert

    "module::nested::foo" { module::nested::foo 0xABAB = } assert
}