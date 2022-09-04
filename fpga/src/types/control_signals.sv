
package mem_write_addr_source_t;
    typedef enum { 
        register_data,
        sp
    } t;
endpackage

package mem_write_data_source_t;
    typedef enum { 
        register_data,
        next_pc,
        this_pc, 
        sr
    } t;
endpackage

package pc_data_source_t;
    typedef enum {
        next_pc,
        register,
        irq,
        mem
    } t;
endpackage

package alu_output_override_t;
    typedef enum {
        none,
        imm8,
        imm8_high
    } t;
endpackage