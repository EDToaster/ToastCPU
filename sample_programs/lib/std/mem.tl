#include<std/io>

mod mem {
    using mem
    using io

    //         dest num
    fn memzero $a* u16 -> {
        swap as(u16*) swap 0 swap memset
    }

    // Sets num elements at dest to val
    // todo: make this a lot cleaner without let bindings
    //        dest val num
    fn memset u16* u16 u16 -> {
        let dest val num {
            0 while { dup num load < } {
                { val load } { over dest load + } store

                1 +
            }
            drop
        }
    }

    fn memdump $a* u16 -> {
        swap as(u16*) swap
        let arr num {
            0 while { dup num load < } {
                { arr load } over + load print_word
                1 +
            }
            drop
        }
    }

    fn booldump $a* u16 -> {
        swap as(u16*) swap
        while { dup 0 > } {
            swap dup load if { '1' } else { '0' } print_char
            1 + swap
            1 -
        }
        drop drop
    }
}
