#include "defs.hpp"

class Memory {
private: 
    union {
        WORD memory_bank[0x10000];
        struct {
            WORD rom[0x8000];
            WORD ram[0x4000];
            WORD io[0x4000];
        };
    } memory;
public:
    Memory() {
        // set 

    }

    WORD *raw() {
        return memory.memory_bank;
    }

    void poke(ADDR addr, WORD value) {
        memory.memory_bank[addr] = value;
    }

    WORD peek(ADDR addr) {
        return memory.memory_bank[addr];
    }
};