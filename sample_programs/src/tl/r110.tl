#include<std/io>

inline WIDTH 99

global buf1 [101] u16 0 // two border cells that are implicitly zero
global buf2 [101] u16 0

global curr_buff u16* 0
global back_buff u16* 0

// swap the values in the two pointers 
fn pswap $a* $a* -> {
    // ap bp b a
    over load over load swap
    // ap b
    rot store
    swap store
}

fn print_dot u16 -> {
    if {
        0x3F20
    } else {
        0x0020
    }
    print_char
}

fn print_buf u16* -> {
    '\n' print_char

    1 while { dup WIDTH <= } {
        over over + load print_dot
        1 + 
    }
    drop drop
}

fn main -> {
    // write 1 at end of buf1
    1 buf1 WIDTH + store

    // setup pointers
    buf1 curr_buff store
    buf2 back_buff store

    curr_buff load print_buf

    while { 1 } {
        // run 1 iteration of the simulation

        // read from curr_buff and write to back_buff
        1 while { dup WIDTH <= } {
            // index for current cell
            
            // ((*curr_buff)[i-1] << 1) i
            curr_buff load over + 1 - load 1 << swap
            // ((*curr_buff)[i] + ((*curr_buff)[i-1] << 1) << 1) i
            curr_buff load over + load rot + 1 << swap
            // i sum
            curr_buff load over + 1 + load rot + ??

            // back_buff[i] = !(sum == 7 || sum == 4 || sum == 0)

            // rule 30
            // dup 0 = over 5 = || over 6 = || over 7 = || ! swap drop

            // rule 90
            // dup 7 = over 5 = || over 2 = || over 0 = || ! swap drop

            // rule 110
            dup 7 = over 4 = || over 0 = || ! swap drop

            // rule 184
            // dup 0 = over 1 = || over 2 = || over 6 = || ! swap drop

            // i new_val

            over back_buff load + store

            1 +
        }
        drop

        // swap buffers
        curr_buff back_buff pswap

        // print
        curr_buff load print_buf
    }    
}