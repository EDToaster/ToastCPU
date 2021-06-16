#include <cstdint>
#include <iostream>

#include "defs.hpp"

typedef enum {
    V, C, N, Z, X
} StatusFlag;

typedef WORD StatusFlagValue;

// 16 bit registers
class Registers {
private:
    WORD bank[8][256];
    WORD general[5];
    WORD stack_pointer;
    union {
        struct {
            uint8_t register_select;
            uint8_t __padding__: 3;
            StatusFlagValue V: 1, C: 1, N: 1, Z: 1, X: 1;
        };
        WORD raw;
    } status;
    WORD program_counter;
public:
    Registers() {
        reset();
    }

    void reset() {
        // registers do not need to be reset, they should 
        // always be initialized manually when needed

        // stack grows up from the start of the stack segment
        stack_pointer = 0x8000;
        status.raw = 0;
        program_counter = 0;

        std::cout << std::hex;
    }

    WORD* get_register(uint8_t select) {
        if (select < 8) {
            uint8_t rbs = status.register_select;
            return &bank[rbs][select];
        } else if (select >= 8 && select < 13) {
            return &general[select - 8];
        } else if (select == 13) {
            return &stack_pointer;
        } else if (select == 14) {
            return &status.raw;
        } else {
            return &program_counter;
        }
    }
    
    void set_status_flag(StatusFlag flag, StatusFlagValue value) {
        switch(flag) {
            case StatusFlag::V:
                status.V = value;
            case StatusFlag::C:
                status.C = value;
            case StatusFlag::N:
                status.N = value;
            case StatusFlag::Z:
                status.Z = value;
            case StatusFlag::X:
                status.X = value;
            default: break;
        }
    }

    StatusFlagValue get_status_flag(StatusFlag flag) {
        switch(flag) {
            case StatusFlag::V:
                return status.V;
            case StatusFlag::C:
                return status.C;
            case StatusFlag::N:
                return status.N;
            case StatusFlag::Z:
                return status.Z;
            case StatusFlag::X:
                return status.X;
            default: return 0;
        }
    } 

    uint8_t* get_register_select() {
        return &status.register_select;
    }

    WORD* get_program_counter() {
        return &program_counter;
    }

    void dump_registers() {
        std::cout << "=== Current Register States ===" << std::endl;
        std::cout << "    General: " << std::endl;
        std::cout << "        R8:  " << general[0] << std::endl;
        std::cout << "        R9:  " << general[1] << std::endl;
        std::cout << "        R10: " << general[2] << std::endl;
        std::cout << "        R11: " << general[3] << std::endl;
        std::cout << "        R12: " << general[4] << std::endl;
        std::cout << "    Stack:   " << stack_pointer << std::endl;
        std::cout << "    Status:  " << std::endl;
        std::cout << "        RS:  " << (unsigned int) *get_register_select() << std::endl;
        std::cout << "       "
                    << " V: " << (unsigned int) get_status_flag(StatusFlag::V)
                    << " C: " << (unsigned int) get_status_flag(StatusFlag::C)
                    << " N: " << (unsigned int) get_status_flag(StatusFlag::N)
                    << " Z: " << (unsigned int) get_status_flag(StatusFlag::Z)
                    << " X: " << (unsigned int) get_status_flag(StatusFlag::X)
                    << std::endl;
        std::cout << "    Program: " << program_counter << std::endl;
        std::cout << "===============================" << std::endl;
    }
};