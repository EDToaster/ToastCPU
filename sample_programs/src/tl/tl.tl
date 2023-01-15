global var1 u16 0xDEAD

// teststr, assertstr, cond
fn assert u16* u16 -> {
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
    drop
}

# multiply by repeated add
fn mul u16 u16 -> u16 {
    0 let a b acc {
        while { b load } {
            # acc += a
            acc load { a load } + acc store
            # b -= 1
            b load 1 - b store
        }
        acc load
    }
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
    "1 !"       { 1   !  0 = }  assert
    "0 !"       { 0   !  1 = }  assert
}

#         strptr  len
fn strlen u16* -> u16 {
    # strptr strptr+13
    dup

    while { dup load } {
        1 +
    }

    swap -
}

fn test_strlen -> {
    "Testing strlen\n" ps
    "strlen(\"abcd\")" { "abcd" strlen 4 = } assert
    "strlen(\"\")" { "" strlen 0 = } assert
}

fn test_let -> {
    "Testing let\n" ps
    11 0xA 0xB let a b {
        { 11 = } "11" swap assert
        "a1" { a load 0xA = } assert
        "b1" { b load 0xB = } assert

        0xC 0xD let b c {
            "a2" { a load 0xA = } assert
            "b2" { b load 0xC = } assert
            "c2" { c load 0xD = } assert
        }
    }
}

fn test_mul -> {
    "Testing mul\n" ps
    "1 x 2" { 1 2 mul 2 = } assert
    "2 x 5" { 2 5 mul 10 = } assert
    "0 x 10" { 0 10 mul 0 = } assert
    "10 x 0" { 10 0 mul 0 = } assert
}

fn test_global -> {
    "Testing global vars\n" ps
    "var1 == 0xDEAD" { var1 load 0xDEAD = } assert
    "var1 = 10" { 10 var1 store var1 load 10 = } assert
}

fn main -> {
    "Hello from ToastLang!\n\n" ps

    test_sum
    test_arith
    test_logical
    test_strlen
    test_let
    test_mul
    test_global

    "Passed All Test Cases!" ps
}

### START PRINT MODULE
# 100 chars by 60 lines

global text_progress u16    100
global text_ptr      u16*   0x0000

fn pc u16 -> {
    dup '\n' = if {
        drop
        text_ptr load text_progress load + text_ptr ?? store
        100 text_progress ?? store
    } else {
        text_ptr load swap

        # print the char
        0b0011100000000000 |
        over store

        1 + text_ptr store

        text_progress load 1 - dup 0 = if {
            drop 100
        }
        text_progress store
    }

    text_ptr load 6000 >= if {
        0 text_ptr store
        100 text_progress store
    }
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
fn ps u16* -> {
    dup
    while { load dup } {
        # str char
        pc 1 + dup
    }
    drop drop
}