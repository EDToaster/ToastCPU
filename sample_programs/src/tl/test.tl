#include<std/io>
#include<std/assert>
#include<std/mem>
#include<std/string>
#include<std/rand>

#include<tests/arith>
#include<tests/color>
#include<tests/global>
#include<tests/inline>
#include<tests/let>
#include<tests/logical>
#include<tests/memset>
#include<tests/mul>
#include<tests/rand>
#include<tests/rec>
#include<tests/string>
#include<tests/struct>
#include<tests/sum>

fn main -> {
    "Hello from ToastLang!\n" print_string

    test_sum
    test_arith
    test_color
    test_logical
    test_strlen
    test_let
    test_streq
    test_mul
    test_global
    test_random
    test_inline
    test_rec
    test_struct
    test_memset

    "Passed All Test Cases!" print_string
}