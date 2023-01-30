#include<std/assert>
#include<std/io>

fn test_logical -> {
    "Testing Logical\n" io::print_string
    "1 && 1"    { 1 1 && 1 = }  assert
    "0 && 1"    { 0 1 && 0 = }  assert
    "0 || 0"    { 0 0 || 0 = }  assert
    "1 || 0"    { 1 0 || 1 = }  assert
    "1 !"       { 1   !  0 = }  assert
    "0 !"       { 0   !  1 = }  assert
}