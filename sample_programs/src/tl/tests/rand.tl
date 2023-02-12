#include<std/io>
#include<std/assert>
#include<std/rand>

mod rand {
    using rand
    using assert
    using io

    global random rand::Random 0

    inline SEED 0xDEAD

    fn test -> {
        "Testing random... " print
        random SEED rand_init

        // period
        1 while { random rand_int SEED = ! } {
            ++
        }

        "Random period = 2^16-1" { over 0xFFFF = } assert drop
    }
}

fn test_rand -> {
    rand::test
}