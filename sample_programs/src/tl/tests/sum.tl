#include<std/assert>
#include<std/io>

mod sum {
    using sum
    using assert
    using io

    // Sums 1..n inclusive
    fn sum u16 -> u16 {
        0 swap
        while {dup} {
            swap over + swap 1 -
        }
        drop
    }

    fn test -> {
        "Testing Sum... " print
        "sum(7) == 21"      { 7 sum 28 -   } assert_false
        "sum(22) == 253"    { 22 sum 253 - } assert_false
    }
}

fn test_sum -> {
    sum::test
}