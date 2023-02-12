#include<std/mem>

mod collection {
    using collection

    // Ring buffer data structure 
    struct RingBuffer {
        buffer  u16*    // pointer to an array at least of size `{size}`
        _mask   u16     // mask of widx and ridx
        _widx   u16     // index into `{buffer}` to next be written to
        _ridx   u16     // index into `{buffer}` to next be read from
    }

    mod RingBuffer {
        fn init 
            RingBuffer* // buffer
            u16*        // rawbuffer
            u16         // n        : 2^n is the size of the buffer
            ->
        {
            1 swap << 1 - rot dup ._mask rot swap store
            swap over .buffer store

            RingBuffer::reset
        }

        fn reset RingBuffer* -> {
            0 over ._widx store
            0 swap ._ridx store 
        }

        fn enqueue 
            RingBuffer* // buffer
            u16         // value
            ->
        {
            over dup .buffer load // buffer val buffer rawbuffer
            swap ._widx load +    // buffer val loc
            store                 // buffer

            // increment widx and use mask
            dup ._widx load 1 + // buffer widx+1
            over ._mask load    // buffer widx+1 mask
            &
            swap ._widx store
        }

        fn dequeue
            RingBuffer* // buffer
            ->
            u16         // value
            bool        // is_valid
        {
            // check if _widx and _ridx are the same
            dup ._widx load over ._ridx load = if {
                drop 0 false
            } else {
                dup .buffer load over ._ridx load + load

                swap dup // val buffer buffer
                ._ridx load 1 + over ._mask load & // val buffer ridx+1
                swap ._ridx store

                true
            }
        }

        // RingBuffer* -> u16
        inline size { ._mask load 1 + }
    }
}