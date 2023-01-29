#include<std/io>
#include<std/assert>
#include<std/rand>


global random Random 0

fn test_random -> {
    "Testing random\n" print_string
    random 0xDEAD rand_init

    // period
    0
    while { random rand_int 0xDEAD = ! } {
        1 +
    }

    "Random period = 2^16-1" { over 0xFFFE = } assert drop
}