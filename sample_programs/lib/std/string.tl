
mod str {
    using str

    // Get the terminal byte of the string
    fn strterm u16* -> u16* {
        while { dup load } {
            ++
        }
    }

    //        strptr  len
    fn strlen u16* -> u16 {
        dup strterm swap -
    }

    //       str1 str2 len
    fn streq u16* u16* u16 -> bool {
        let len {
            while { len load 0 > } {
                over load over load = ! if {
                    drop drop false return
                    // todo: fix this since everything after return is unreachable
                    drop 0 as(u16*) 0 as(u16*)
                }

                ++ swap ++ swap
                len load -- len store
            }
        }
        drop drop true
    }
}