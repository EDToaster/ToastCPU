#include<std/io>
#include<std/mem>
#include<std/string>

fn isr -> {
    io::keyboard_handle_input
}

fn main -> {
    io::keyboard_init
    game::start_game
}

mod game {
    using game

    // using struct and module of the same name lets
    // you define methods for the struct. Similar
    // to `impl`s for rust.
    struct Game {
        guessed_letters [26]    u16

        word                    u16*
        word_length             u16

        // max word length is 16
        correct_letters [16]    u16

        // max incorrect guesses is 6
        incorrect_guesses       u16
    }

    mod Game {
        fn init Game* u16* -> {
            // set everything to zero
            swap dup sizeof(Game) mem::memzero

            // store nonzero values
            over str::strlen over .word_length store
            .word store
        }

        //       game  ascii
        fn guess Game* u16 -> {
            // check if letter is valid
            dup 'a' < over 'z' > || if {
                drop drop return
                // todo : fix this because `return` is buggy
                0 as(Game*) 0
            }

            // check if we already guessed this letter and save it to the guessed letters list
            over .guessed_letters over 'a' - + dup load if {
                drop drop drop return
                // todo : fix this because `return` is buggy
                0 as(Game*) 0 0 as(u16*)
            } 1 swap store

            0 let game guess correct {
                0 while { dup game load .word_length load < } {
                    dup game load .word load + load guess load = if {
                        1 correct store

                        // mark the letter correct
                        dup game load .correct_letters + 1 swap store
                    }

                    1 + 
                } drop
                game load correct load
            }

            // game guess
            ! if {
                dup .incorrect_guesses load 1 + over .incorrect_guesses store
            }

            drop
        }

        fn print_game Game* -> {
            let game {
                // print the word
                0 while { dup game load .word_length load < } {
                    dup game load .correct_letters + load if {
                        dup game load .word load + load
                    } else {
                        '_' 
                    } io::print_char
                    1 + 
                } drop

                // print number of wrong guesses
                "   Wrong guesses: " io::print_string game load .incorrect_guesses load io::print_decimal "\n\n" io::print_string
            }
        }
    }

    
    global game Game 0

    fn start_game -> {
        game "absentmindedness" Game::init
        game Game::print_game

        while { 1 } {
            while { io::keyboard_get_input dup } {
                game over Game::guess drop
                game Game::print_game
            } drop
        }
    }
}