
mod math {

    // multiply by repeated add
    fn mul u16 u16 -> u16 {
        0 let a b acc {
            while { b load } {
                // acc += a
                acc load { a load } + acc store
                // b -= 1
                b load 1 - b store
            }
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