
#include<std/assert>
#include<std/io>

mod using_module {
    using using_module

    mod nested {
        using using_module::nested
        
        fn bar -> u16 {
            foo 0x4321 + 
        }
    }

    fn foo -> u16 {
        0x5678
    }
}

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

using using_module 
using using_module::nested

using io

fn test_modules -> {
    "Testing Modules... " print
    
    "Inline val"                { module::INLINE_VAL 0xDEAD =  } assert::assert
    "Inline block"              { module::INLINE_BLOCK 3 =     } assert::assert

    module::g module::init_struct drop

    "module::foo"               { module::foo 0xBEEF =         } assert::assert
    "module::g .a load"         { module::g .a load 0xBEEF =   } assert::assert

    "module::nested::foo"       { module::nested::foo 0xABAB = } assert::assert

    "using_module::foo"         { foo 0x5678 =                 } assert::assert
    "using_module::nested::bar" { bar 0x9999 =                 } assert::assert
}
