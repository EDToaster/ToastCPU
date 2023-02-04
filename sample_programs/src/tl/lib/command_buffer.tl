#include<std/mem>

mod command_buffer {
    using command_buffer
    using mem

    inline COMMAND_BUFFER_SIZE 32
    struct CommandBuffer {
        offset      u16
        // todo: add support for inlines in struct defs by changing IntLiteral to Statement
        command[32] u16
        _nullterm   u16
    }
    
    mod CommandBuffer {
        fn push_char CommandBuffer* u16 -> {
            let buf c {
                buf load .offset load { dup 32 < } if {
                    buf load .command + 
                    c load swap
                    store

                    buf load .offset dup load 1 + swap store
                } else { drop }
            }
        }

        fn reset CommandBuffer* -> {
            0 over .offset store
            dup .command 0 COMMAND_BUFFER_SIZE memset
            0 over ._nullterm store
            drop
        }
    }
}