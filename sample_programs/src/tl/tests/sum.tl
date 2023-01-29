#include<std/assert>
#include<std/io>

// Sums 1..n inclusive
fn sum u16 -> u16 {
    0 swap
    while {dup} {
        swap over + swap 1 -
    }
    drop
}

fn test_sum -> {
    "Testing Sum\n" print_string
    "sum(7) == 21"      { 7 sum 28 - 0 = }      assert
    "sum(22) == 253"    { 22 sum 253 - 0 = }    assert
}