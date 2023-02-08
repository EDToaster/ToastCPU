#include<std/io>
#include<std/mem>
#include<std/string>
#include<std/rand>

#include<tests/arith>
#include<tests/bool>
#include<tests/color>
#include<tests/collection>
#include<tests/func_ptr>
#include<tests/global>
#include<tests/inline>
#include<tests/let>
#include<tests/logical>
#include<tests/memset>
#include<tests/modules>
#include<tests/mul>
#include<tests/rand>
#include<tests/rec_types>
#include<tests/rec>
#include<tests/string>
#include<tests/struct>
#include<tests/sum>

mod test_suite {
    using test_suite
    using io

    // room for 32 tests
    struct TestSuite {
        ptrs [32] (->)*
        size u16
    }

    global TESTS TestSuite 0

    fn push_test (->)* -> {
        TESTS .ptrs TESTS .size load + store
        TESTS .size dup load 1 + swap store
    }

    fn run_tests -> u16 {
        0 while { dup TESTS .size load < } {
            TESTS .ptrs over + load over run_test
            1 +
        }
    }

    fn run_test (->)* u16 -> {
        1 + print_decimal ": " print_string
        4 left_align ()
        get_style GREEN fg 35 left_align "Passed\n" print_string set_style
    }

    fn main -> {
        BLACK bg WHITE fg
        clear

        ptr(test_color)         push_test
        ptr(test_sum)           push_test
        ptr(test_arith)         push_test
        ptr(test_logical)       push_test
        ptr(test_strlen)        push_test
        ptr(test_let)           push_test
        ptr(test_streq)         push_test
        ptr(test_mul)           push_test
        ptr(test_global)        push_test
        ptr(test_rand)          push_test
        ptr(test_inline)        push_test
        ptr(test_rec)           push_test
        ptr(test_struct)        push_test
        ptr(test_memset)        push_test
        ptr(test_modules)       push_test
        ptr(test_func_ptr)      push_test
        ptr(test_rec_types)     push_test
        ptr(test_bool)          push_test
        ptr(test_collection)    push_test

        // run all test cases
        run_tests
        
        '\n' print_char
        GREEN fg "Passed " print_string print_decimal " Test Cases!" print_string

        "\n\n\n" print_string

        WHITE bg BLACK fg
        8 left_align "               __        \n" print_string
        8 left_align "              / _)       \n" print_string
        8 left_align "     _.----._/ /         \n" print_string
        8 left_align "   _|         /          \n" print_string
        8 left_align " _|  (  | (  |           \n" print_string
        8 left_align "/__.-'|_|--|_| edtoaster \n" print_string
    }
}

fn main -> {
    test_suite::main
}