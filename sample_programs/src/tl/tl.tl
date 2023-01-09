
fn fib u16 u16 -> u16 u16 {
    swap dup2 + swap drop
}

fn main -> {
    1 1

    dup p fib
    dup p fib
    dup p fib
    dup p fib
}
