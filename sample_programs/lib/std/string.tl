//         strptr  len
fn strlen u16* -> u16 {
    // strptr strptr+13
    dup
    while { dup load } {
        1 +
    }
    swap -
}