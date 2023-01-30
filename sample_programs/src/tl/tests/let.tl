#include<std/assert>
#include<std/io>

fn test_let -> {
    "Testing let\n" io::print_string
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