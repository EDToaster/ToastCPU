
mod math {
    using math

    //           acc a   b
    fn _mul_iter u16 u16 u16 -> u16 u16 u16 {
        over 1 & if { 
            rot over + // a b acc
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
    }

    fn divmod u16 u16 -> u16 u16 {
        let num denom {
            0
            while { num load denom load >= } {
                num load denom load - num store
                ++
            }
            num load
        }
    }
}