#include<std/io>

using io 

fn do_something -> {
    "Hello!\n" print_string
}

fn main -> {
    clear
    ptr(do_something) dup () dup () drop
}