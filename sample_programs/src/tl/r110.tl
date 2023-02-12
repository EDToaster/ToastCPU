#include<std/io>
#include<std/rand>
#include<std/types>

mod r110 {
    using r110
    using io
    using rand

    inline WIDTH 100

    global buf1 [102] bool 0 // two border cells that are implicitly zero
    global buf2 [102] bool 0

    global curr_buff bool* 0
    global back_buff bool* 0

    // swap the values in the two pointers 
    fn pswap $a* $a* -> {
        // ap bp b a
        over load over load swap
        // ap b
        rot store
        swap store
    }

    global rand Random 0

    //              seed arr
    fn randomize_buf u16 bool* -> {
        rand 0x1D2D rand_init

        let n buf {
            0 while { dup n load < } {
                rand rand_int bool::from_u16 buf load store

                buf load ++ buf store
                ++
            }
            drop
        }
    }

    fn print_dot bool -> {
        if {
            0x3F20
        } else {
            0x0020
        }
        print_char
    }

    fn print_buf bool* -> {
        1 while { dup WIDTH <= } {
            over over + load print_dot
            ++
        }
        drop drop
    }

    inline RANDOMIZE    true
    inline BOUND_LEFT   true
    inline BOUND_RIGHT  true

    fn r110_main -> {
        
        // write 1 at end of buf1
        RANDOMIZE if  {
            WIDTH 2 + buf1 randomize_buf
        } else {
            true buf1 WIDTH + store
        }

        BOUND_LEFT buf1 store 
        BOUND_LEFT buf2 store
        BOUND_RIGHT buf1 WIDTH 1 + + store 
        BOUND_RIGHT buf2 WIDTH 1 + + store 

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
                curr_buff load over + -- load as(u16) 1 << swap
                // ((*curr_buff)[i] + ((*curr_buff)[i-1] << 1) << 1) i
                curr_buff load over + load as(u16) rot + 1 << swap
                // i sum
                curr_buff load over + ++ load as(u16) rot + ??

                // back_buff[i] = !(sum == 7 || sum == 4 || sum == 0)

                // rule 30
                // dup 0 = over 5 = || over 6 = || over 7 = || ! swap drop

                // rule 90
                dup 7 = over 5 = || over 2 = || over 0 = || ! swap drop

                // rule 110
                // dup 7 = over 4 = || over 0 = || ! swap drop

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
}

fn main -> {
    r110::r110_main
}