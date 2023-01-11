// teststr, assertstr, cond
fn assert u16 u16 -> {
    if {
        #"Passed\n" ps
    } else {
        dup ps
        ' ' pc
        "Assert Failed\n" ps
        halt
    }
    drop
}

// Sums 1..n inclusive
fn sum u16 -> u16 {
    0 swap
    while {dup} {
        swap over + swap 1 -
    }

    drop # the counter which is 0 at this point

    //a 1
    //1 a

    //loop
    //a 1
    //a 1 a
    //a 1+a
    //1+a a
    //1+a a-1
}

fn test_sum -> {
    "Testing Sum\n" ps
    "sum(7) == 21"      { 7 sum 28 - 0 = }      assert
    "sum(22) == 253"    { 22 sum 253 - 0 = }    assert
}

fn test_arith -> {
    "Testing Arith\n" ps
    "7 = 7"     { 7 7 = 1 = }   assert
    "6 = 7"     { 6 7 = 0 = }   assert
    "6 > 7"     { 6 7 > 0 = }   assert
    "7 > 7"     { 7 7 > 0 = }   assert
    "8 > 7"     { 8 7 > 1 = }   assert
    "6 >= 7"    { 6 7 >= 0 = }  assert
    "7 >= 7"    { 7 7 >= 1 = }  assert
    "8 >= 7"    { 8 7 >= 1 = }  assert
    "6 < 7"     { 6 7 < 1 = }   assert
    "7 < 7"     { 7 7 < 0 = }   assert
    "8 < 7"     { 8 7 < 0 = }   assert
    "6 <= 7"    { 6 7 <= 1 = }  assert
    "7 <= 7"    { 7 7 <= 1 = }  assert
    "8 <= 7"    { 8 7 <= 0 = }  assert
}

fn test_logical -> {
    "Testing Logical\n" ps
    "1 && 1"    { 1 1 && 1 = }  assert
    "0 && 1"    { 0 1 && 0 = }  assert
    "0 || 0"    { 0 0 || 0 = }  assert
    "1 || 0"    { 1 0 || 1 = }  assert
    "1 !"       { 1 ! 0 = }     assert
    "0 !"       { 0 ! 1 = }     assert
}

fn main -> {
    "Hello from ToastLang!\n\n" ps

    test_sum
    test_arith
    test_logical

    "Passed All Test Cases!" ps

    fib
}

#         strptr len
fn strlen u16 -> u16 {
    0 over
    # strptr acc strptr
    while { load dup } {
        drop 1 + swap 1 + swap over
    }
    drop swap drop
}

# Print the hex char
fn ph u16 -> {
    dup 10 < if {
        '0' + pc
    } else {
        10 - 'A' + pc
    }
}

# Print the nibble
#     val
fn pn u16 -> {
    dup 4 >> 0xF & ph
    dup 0xF & ph
    drop
}

# Print the word
#    val
fn p u16 -> {
    dup 8 >> 0xFF & pn
    dup 0xFF & pn
    drop
}

#     strptr
fn ps u16 -> {
    dup
    while { load dup } {
        # str char
        pc 1 + dup
    }
    drop drop
}

fn fib -> {
    "Testing fib sequence ...\n" ps

    # a b
      0 1
    while { over 10000 < } {
        over p ' ' pc
        swap over +
    }

    drop drop

    0xDEAD p 0xBEEF p
}