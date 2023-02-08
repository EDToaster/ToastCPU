#include<std/io>
#include<std/assert>
#include<std/collection>

mod collection {
    using collection
    using io
    using assert
    
    global buffer               RingBuffer  0
    global internal_buffer [32] u16         0

    fn test -> {
        "Testing collections..." print_string

        buffer internal_buffer 5 RingBuffer::init

        "Buffer mask" { buffer ._mask load 0x001F =  }  assert
        "Buffer size" { buffer RingBuffer::size 32 = }  assert

        buffer 0xDEAD RingBuffer::enqueue
        buffer 0xBEEF RingBuffer::enqueue

        buffer RingBuffer::dequeue "Dequeue valid"      swap assert 0xDEAD = "Dequeue value" swap assert
        buffer RingBuffer::dequeue "Dequeue valid"      swap assert 0xBEEF = "Dequeue value" swap assert
        buffer RingBuffer::dequeue "Dequeue invalid"    swap assert_false drop
    }
}

fn test_collection -> {
    collection::test
}