#include<std/assert>
#include<std/io>
#include<std/string>

fn test_strlen -> {
    "Testing strlen\n" io::print_string
    "strlen(\"abcd\")" { "abcd" strlen 4 = } assert
    "strlen(\"\")" { "" strlen 0 = } assert
}

fn test_streq -> {
    "Testing streq\n" io::print_string
    "streq(\"\", \"\", 0)" { "" "" 0 streq } assert
    "streq(\"a\", \"\", 0)" { "a" "" 0 streq } assert
    "streq(\"a\", \"a\", 1)" { "a" "a" 1 streq } assert
    "streq(\"a\", \"b\", 1)" { "a" "b" 1 streq 0 = } assert
    "streq(\"aaa\", \"aaab\", 3)" { "aaa" "aaab" 3 streq } assert
    "streq(\"aaa\", \"aaab\", 4)" { "aaaa" "aaab" 4 streq 0 = } assert
}
