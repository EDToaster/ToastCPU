
mod math {
    using math

    // multiply by shift add
    fn mul u16 u16 -> u16 {
        0 let acc {
            // a, b
            // if a & 1, add b
            unroll 16 {
                over 1 & if { dup acc load + acc store } 1 << swap 1 >>> swap
            }

            drop drop
            acc load
        }
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