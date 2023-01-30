#include<std/assert>
#include<std/io>

using assert
using io

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