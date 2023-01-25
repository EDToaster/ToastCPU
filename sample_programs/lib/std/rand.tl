
struct Random {
    state u16
}

//           rand    seed
fn rand_init Random* u16 -> {
    over .state store drop
}

fn rand_int Random* -> u16 {
    dup .state load
    // Random* u16
    dup 7   >>> ^
    dup 9   <<  ^
    dup 13  >>> ^
    swap over
    // u16 Random* u16
    swap .state store
}