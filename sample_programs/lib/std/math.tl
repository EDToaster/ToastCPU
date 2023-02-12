
mod math {
    using math

    //           acc a   b
    fn _mul_iter u16 u16 u16 -> u16 u16 u16 {
        over 1 & if { 
            rot over + // b a acc
        } else {
            rot
        }
        rot 1 >>>
        rot 1 <<
    }

    // multiply by shift add
    fn mul u16 u16 -> u16 {
        0 rot rot // acc, a, b

        while { dup } {
            _mul_iter
        }

        drop drop

        // 0 let acc {
        //     // a, b
        //     // if a & 1, add b
        //     unroll 16 {
        //         over 1 & if { dup acc load + acc store } 1 << swap 1 >>> swap
        //     }

        //     drop drop
        //     acc load
        // }
    }

    fn divmod u16 u16 -> u16 u16 {
        let num denom {
            0
            while { num load denom load >= } {
                num load denom load - num store
                1 +
            }
            num load
        }
    }
}