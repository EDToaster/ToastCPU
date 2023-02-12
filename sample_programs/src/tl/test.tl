#include<std/io>
#include<std/mem>
#include<std/string>
#include<std/rand>
#include<std/iter>

#include<tests/arith>
#include<tests/bool>
#include<tests/bugs>
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
#include<tests/intern>

mod test_suite {
    using test_suite
    using io

    // room for 32 tests
    struct TestSuite {
        ptrs [32] (->)*
        size u16
    }

    mod TestSuite {
        using test_suite::TestSuite

        fn push TestSuite* (->)* -> {
            over dup .ptrs  // suite ptr suite ptrs
            swap .size load // suite ptr ptrs size
            + store         // suite 
            
            dup .size load ++ swap .size store
        }

        fn run_all TestSuite* -> u16 {
            0 while { over .size load over > } {
                over .ptrs over + load over run_test
                ++
            } swap drop
        }

        fn run_test (->)* u16 -> {
            ++ print_decimal ": " print
            4 left_align ()
            get_style GREEN fg 35 left_align "Passed\n" print set_style
        }
    }

    global suite        TestSuite 0
    global impl_details TestSuite 0 

    fn main -> {

        suite ptr(test_color)         TestSuite::push
        suite ptr(test_sum)           TestSuite::push
        suite ptr(test_arith)         TestSuite::push
        suite ptr(test_logical)       TestSuite::push
        suite ptr(test_strlen)        TestSuite::push
        suite ptr(test_let)           TestSuite::push
        suite ptr(test_streq)         TestSuite::push
        suite ptr(test_mul)           TestSuite::push
        suite ptr(test_global)        TestSuite::push
        suite ptr(test_rand)          TestSuite::push
        suite ptr(test_inline)        TestSuite::push
        suite ptr(test_rec)           TestSuite::push
        suite ptr(test_struct)        TestSuite::push
        suite ptr(test_memset)        TestSuite::push
        suite ptr(test_modules)       TestSuite::push
        suite ptr(test_func_ptr)      TestSuite::push
        suite ptr(test_rec_types)     TestSuite::push
        suite ptr(test_bool)          TestSuite::push
        suite ptr(test_collection)    TestSuite::push

        impl_details ptr(test_intern) TestSuite::push

        clear

        BLACK bg WHITE fg
        suite TestSuite::run_all
        '\n' print_char GREEN fg "Passed " print print_decimal " Test Cases!\n" println

        BLACK bg WHITE fg
        impl_details TestSuite::run_all
        '\n' print_char GREEN fg "Passed " print print_decimal " Implementation Test Cases!\n" println

        "\n\n" print

        WHITE bg BLACK fg
        8 left_align "               __        " println
        8 left_align "              / _)       " println
        8 left_align "     _.----._/ /         " println
        8 left_align "   _|         /          " println
        8 left_align " _| .(  | (  |           " println
        8 left_align "/_.' '|_|--|_| edtoaster " println
    }
}

fn main -> {
    test_suite::main
}