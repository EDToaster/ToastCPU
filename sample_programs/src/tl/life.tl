#include<std/mem>
#include<std/io>
#include<std/rand>

mod life {
    using life

    inline WIDTH_POW    5
    inline HEIGHT_POW   4
    inline WIDTH        32
    inline HEIGHT       16
    inline SIZE         512

    global buf1 [512]  bool   0
    global buf2 [512]  bool   0

    global front_buf    bool*  0 
    global back_buf     bool*  0

    global random       rand::Random 0


    // swap the values in the two pointers 
    fn pswap $a* $a* -> {
        // ap bp b a
        over load over load swap
        // ap b
        rot store
        swap store
    }

    // we loop all the sides so the
    // manifold is a torus
    //          x   y      ind
    fn index_of u16 u16 -> u16 {
        // mod the width and heights 
        swap WIDTH -- & swap HEIGHT -- &

        // ind = y * WIDTH + x
        WIDTH_POW << +
    }

    fn state_of u16 u16 -> bool {
        index_of front_buf load + load
    }

    fn print_row bool* -> bool* {
        0 while { dup WIDTH < } {
            swap // count buf
            dup load if {
                io::WHITE
            } else {
                io::BLACK
            }
            
            io::bg ' ' io::print_char
            ++ swap ++
        } drop
    }

    fn print_buf -> {
        io::print_reset

        front_buf load 
        0 while { dup HEIGHT < } {
            swap // count buf
            print_row '\n' io::print_char swap
            ++
        } drop drop
    }

    fn init u16 -> {
        // init random
        random swap rand::rand_init

        // fill buf1 with random noise
        0 while { dup SIZE < } {
            buf1 over + random rand::rand_int 1 & as(bool) swap store
            ++
        } drop

        // set buffer pointers
        buf1 front_buf store
        buf2 back_buf  store
        
        print_buf
    }

    // get the number of neighbours at x and y
    fn num_neighbours u16 u16 -> u16 {
        let x y {
            0 
            x load -- y load -- state_of as(u16) +
            x load    y load -- state_of as(u16) +
            x load ++ y load -- state_of as(u16) +
            x load -- y load    state_of as(u16) +
            x load ++ y load    state_of as(u16) +
            x load -- y load ++ state_of as(u16) +
            x load    y load ++ state_of as(u16) +
            x load ++ y load ++ state_of as(u16) +
        }
    }


    fn iter -> {
        
        0 while { dup WIDTH < } {
            0 while { dup HEIGHT < } {
                over over over over num_neighbours
                rot rot state_of if {
                    // x, y, neighbours
                    dup 2 = swap 3 = ||
                } else {
                    // x, y, neighbours
                    3 =
                }
                // x, y, is_alive
                let is_alive {
                    over over index_of back_buf load + is_alive load swap store
                }
                ++
            } drop
            ++
        } drop

        // swap buffers
        back_buf front_buf pswap
        print_buf
    }
}


fn isr -> {
    io::keyboard_handle_input
}

fn main -> {
    
    io::keyboard_init

    // 93 at 32x16 produces:
    // Shaceship glider at 2c/5 speed.
    //  x x
    // x
    // x   x
    // x
    // x  x
    // xxx
    
    93 dup life::init

    while { io::keyboard_get_input swap drop ! } {

        // { io::keyboard_get_input swap drop } if {
        //     ++ dup life::init
        // }

        life::iter
    }

    drop
}