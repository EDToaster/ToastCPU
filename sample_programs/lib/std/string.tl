
mod str {
    using str

    //         strptr  len
    fn strlen u16* -> u16 {
        // strptr strptr+13
        dup
        while { dup load } {
            1 +
        }
        swap -
    }


    //       str1 str2 len
    fn streq u16* u16* u16 -> u16 {
        let len {
            while { len load 0 > } {
                over load over load = ! if {
                    drop drop 0 return
                    // todo: fix this since everything after return is unreachable
                    drop 0 as(u16*) 0 as(u16*)
                }

                1 + swap 1 + swap
                len load 1 - len store
            }
        }
        drop drop 1
    }
}