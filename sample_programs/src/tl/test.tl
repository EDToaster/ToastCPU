#include<std/io>
#include<std/string>

// teststr, assertstr, cond
fn assert u16* u16 -> {
    if {
        //"Passed\n" ps
    } else {
        dup print_string
        ' ' print_char
        "Assert Failed\n" print_string
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

fn test_sum -> {
    "Testing Sum\n" print_string
    "sum(7) == 21"      { 7 sum 28 - 0 = }      assert
    "sum(22) == 253"    { 22 sum 253 - 0 = }    assert
}

fn test_arith -> {
    "Testing Arith\n" print_string
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
    "Testing Logical\n" print_string
    "1 && 1"    { 1 1 && 1 = }  assert
    "0 && 1"    { 0 1 && 0 = }  assert
    "0 || 0"    { 0 0 || 0 = }  assert
    "1 || 0"    { 1 0 || 1 = }  assert
    "1 !"       { 1   !  0 = }  assert
    "0 !"       { 0   !  1 = }  assert
}


fn test_strlen -> {
    "Testing strlen\n" print_string
    "strlen(\"abcd\")" { "abcd" strlen 4 = } assert
    "strlen(\"\")" { "" strlen 0 = } assert
}

fn test_streq -> {
    "Testing streq\n" print_string
    "streq(\"\", \"\", 0)" { "" "" 0 streq } assert
    "streq(\"a\", \"\", 0)" { "a" "" 0 streq } assert
    "streq(\"a\", \"a\", 1)" { "a" "a" 1 streq } assert
    "streq(\"a\", \"b\", 1)" { "a" "b" 1 streq 0 = } assert
    "streq(\"aaa\", \"aaab\", 3)" { "aaa" "aaab" 3 streq } assert
    "streq(\"aaa\", \"aaab\", 4)" { "aaaa" "aaab" 4 streq 0 = } assert
}

fn test_let -> {
    "Testing let\n" print_string
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
    "Testing mul\n" print_string
    "1 x 2"  { 1 2 mul 2 = }  assert
    "2 x 5"  { 2 5 mul 10 = } assert
    "0 x 10" { 0 10 mul 0 = } assert
    "10 x 0" { 10 0 mul 0 = } assert
}

global var1 u16 0xDEAD
fn test_global -> {
    "Testing global vars\n" print_string
    "var1 == 0xDEAD" { var1 load 0xDEAD = }           assert
    "var1 = 10"      { 10 var1 store var1 load 10 = } assert
}

fn main -> {
    "Hello from ToastLang!\n" print_string

    test_sum
    test_arith
    test_logical
    test_strlen
    test_streq
    test_let
    test_mul
    test_global

    "Passed All Test Cases!" print_string
}