#include <iostream>
#include <cstring>

#include "defs.hpp"
#include "registers.hpp"
#include "memory.hpp"

int handle_arith(uint8_t rx1, uint8_t rx2, uint8_t arith_code, Registers *reg) {
    // handle b1000
    uint16_t r1 = (uint16_t) *reg->get_register(rx1);
    uint16_t r2 = arith_code == 0 ? 0 : (uint16_t) *reg->get_register(rx2);
    // input most significant bit
    uint8_t m1 = (r1 >> 15) & 0x01, m2 = (r2 >> 15) & 0x01;
    uint32_t agg;
    
    switch (arith_code) {
        case 0x0:
            agg = ~r1; break;
        case 0x1:
            agg = r1 & r2; break;
        case 0x2:
            agg = r1 | r2; break;
        case 0x3:
            agg = r1 ^ r2; break;
        case 0x4:
            agg = r1 + r2; break;
        case 0x5:
            agg = r1 - r2; break;
        case 0xC:
            agg = r1 + 1; break;
        case 0xD:
            agg = r1 - 1; break;
        default: // invalid
            return 1;
    }
    // agg most significant bit
    uint8_t ma = (agg >> 15) & 0x01;
    StatusFlagValue overflow_flag = (m1 == m2) && (m1 ^ ma);
    StatusFlagValue carry_flag = (agg >> 16) & 0x01;
    StatusFlagValue negative_flag = (StatusFlagValue) ma;
    StatusFlagValue zero_flag = (agg & 0xFFFF) == 0x0000;
    StatusFlagValue ffff_flag = (agg & 0xFFFF) == 0xFFFF;

    // set flags based on arith code.
    if (arith_code >= 0x4) {
        // needs overflow (V) and carry (C)
        reg->set_status_flag(StatusFlag::V, overflow_flag);
        reg->set_status_flag(StatusFlag::C, carry_flag);
    }
    reg->set_status_flag(StatusFlag::N, negative_flag);
    reg->set_status_flag(StatusFlag::Z, zero_flag);
    reg->set_status_flag(StatusFlag::X, ffff_flag);

    // latch value back into rx1
    *reg->get_register(rx1) = agg & 0xFFFF;

    return 0;
}

#define PROGRAM_LENGTH 256

int main() {

    WORD rom[PROGRAM_LENGTH] = {
        0x3801,                 // imov  r8   0x01
        0x3901,                 // imov  r9   0x01
        0x3A0F,                 // imov  r10  0x0F  // iterations
        0x3C03,                 // imov  r12  0x03  // jump address
        0x2B80,                 // mov   r11  r8
        0x2890,                 // mov   r8   r9
        0x849B,                 // mov   r9   r11
        0x89A0,                 // dec   r10
        0x4C80,                 // jnz   r12
        0x7000                  // halt
    };

    Registers *reg = new Registers();
    reg->dump_registers();
    Memory *mem = new Memory();
    memcpy(mem->raw(), rom, PROGRAM_LENGTH * sizeof(WORD));
    
    
    bool halt = 0;
    while (!halt) {
        WORD instruction = mem->peek(*reg->get_program_counter());
        uint8_t opcode = instruction >> 12;
        uint8_t s1 = (instruction >> 8) & 0xF, s2 = (instruction >> 4) & 0xF, s3 = instruction & 0xF; 
        int8_t imm8 = (s2 << 4) | s3;
        bool negate = (s2 >> 3) & 0x1;
        bool do_jump = negate != reg->get_status_flag(StatusFlag::Z);

        switch (opcode) {
            case 0x0:
                // load
                *reg->get_register(s1) = mem->peek(*reg->get_register(s2));
                break;
            case 0x1:
                // str
                mem->poke(*reg->get_register(s1), *reg->get_register(s2));
                break;
            case 0x2:
                // mov
                *reg->get_register(s1) = *reg->get_register(s2);
                break;
            case 0x3:
                // imov
                *reg->get_register(s1) = imm8;
                break;
            case 0x4:
                // jz
                if (do_jump) 
                    *reg->get_program_counter() = *reg->get_register(s1);
                break;
            case 0x8:
                if (handle_arith(s2, s3, s1, reg)) halt = 1; 
                break;
            case 0x9:
                if (s1 == 0x0) {
                    // inc page
                    (*reg->get_register_select())++;
                } else if (s1 == 0x1) {
                    // dec page
                    (*reg->get_register_select())--;
                } else {
                    halt = 1;
                }
                break;
            default:
                // everything else is a halt
                halt = 1;
                break;

        }

        if (opcode != 0x4 || !do_jump) {
            // advance the program counter
            (*reg->get_program_counter())++;
        }
    }

    reg->dump_registers();

    return 0;
}