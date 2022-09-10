`include "types/control_signals_imports.svh"

module controlpath(
    input logic clock,
    input logic reset, // reset low
    //input logic do_halt,
    input logic Z,
    input logic N,
    
    input logic irq,
    
    input logic [15:0] instruction,
    
    output logic reg_write,
    output logic mem_to_reg, 			// transfer memory to reg? (for load, etc)
    output logic mem_read_is_pc,	// on clock
    output logic mem_read_is_sp,	// on clock

    output alu_output_override_t::t alu_output_override,

    output logic alu_override_imm4,		// override input b of alu to be imm4 value?
    output logic alu_set_flags,		// on clock, set status flags?
    output logic set_pc,					// on clock
    output pc_data_source_t::t pc_data_source,

    output logic sr_from_mem,
    output logic reset_irq,				// todo: posedge clock reset IRQ line (currently async)
    
    output logic set_sp,
    output logic increase_sp,
    
    output logic mem_write,
    output mem_write_addr_source_t::t mem_write_addr_source,
    output mem_write_data_source_t::t mem_write_data_source,
        
    output logic [9:0] state
);
    // halt is set on ALU fault or halt command
    typedef enum { 
        reset_state, 				// reset
        fetch_set_addr, 			// set pc into rom addr
        fetch_set_instruction, 	// fetch instruction into registers
        
        op_decode,
        
        op_load_set_addr,
        op_load_set_register,
        
        op_store,
        
        op_push_store,
        op_push_inc,
        
        op_pop_dec,
        op_pop_set_addr,
        op_pop_set_register,
        
        // jmpl
        op_jmp_link,
        op_jmp_link_inc,

        // jmp
        op_jmp,
        
        // jmpr
        op_jmp_ret_dec,
        op_jmp_ret_set_addr,
        op_jmp_ret_set_pc,

        // irq
        op_irq_jmp_link,		// write PC to stack
        op_irq_jmp_link_inc,	// inc stack
        op_irq_jmp_status,		// write SR to stack
        op_irq_jmp_status_inc,	// inc stack
        op_irq_jmp,				// set PC from irq register
        op_irq_reset,			// reset irq flag

        // rti
        op_rti_dec,				// decrement SP	
        op_rti_set_addr,		// set mem addr to SP
        op_rti_set_sr,			// write stack to SR
        
        op_alu,

        op_imov,
        
        op_halt 					// halt
    } cpu_state;
    
    cpu_state curr_state, next_state;
    
    wire [3:0] opcode = instruction[15:12];
    wire [3:0] aluop = instruction[3:0];
    wire [3:0] jop = instruction[3:0];
    wire override_b = instruction[3];
    wire link_jump = instruction[4];
    wire ret_jump = instruction[5];
    
    assign state = {
        (curr_state == reset_state), 
        (curr_state == fetch_set_addr), 
        (curr_state == fetch_set_instruction), 
        (curr_state == op_decode), 
        (curr_state == op_load_set_addr), 
        (curr_state == op_load_set_register), 
        (curr_state == op_store), 
        (curr_state == op_jmp),
        (curr_state == op_alu), 
        (curr_state == op_halt)
    };
    
    always_ff @(posedge clock or negedge reset) begin: reset_logic
        if (~reset) 
        begin
            // reset logic
            curr_state = reset_state;
        end else begin
            // clocked logic
            curr_state = next_state;
        end
    end
    
    localparam 
        jmp = 4'b0000,
        jz	 = 4'b0001,
        jnz = 4'b0010,
        jn  = 4'b0011,
        jp  = 4'b0100;
    
    wire do_jump;
    // set control signals
    always_comb begin: control_signals
    
        case (jop)
            jmp: 	do_jump = 1'b1;
            jz: 	do_jump = Z;
            jnz: 	do_jump = ~Z;
            jn: 	do_jump = N;
            jp: 	do_jump = ~Z & ~N;
            default: do_jump = 1'b0;
        endcase
        
        // all control flags by default are 0
        reg_write = 1'b0;
        mem_to_reg = 1'b0;
        alu_output_override = alu_output_override_t::none;
        alu_override_imm4 = 1'b0;
        alu_set_flags = 1'b0;
        set_pc = 1'b0;

        pc_data_source = pc_data_source_t::next_pc;

        sr_from_mem = 1'b0;
        mem_write = 1'b0;
        mem_read_is_pc = 1'b0;
        mem_read_is_sp = 1'b0;
        
        mem_write_addr_source = mem_write_addr_source_t::register_data;
        mem_write_data_source = mem_write_data_source_t::register_data;

        set_sp = 1'b0;
        increase_sp = 1'b0;
        reset_irq = 1'b0;
        
        unique case(curr_state)
            reset_state:;
            
            fetch_set_addr,
            fetch_set_instruction: mem_read_is_pc = 1'b1;
            
            op_decode:;
            
            op_load_set_addr, op_pop_set_addr:;
            op_jmp_ret_set_addr, op_rti_set_addr: begin
                mem_read_is_sp = 1'b1;
            end

            op_load_set_register, op_pop_set_register: begin
                reg_write = 1'b1;
                mem_to_reg = 1'b1;
                set_pc = 1'b1;
            end
            
            op_rti_set_sr: begin
                mem_read_is_sp = 1'b1;
                sr_from_mem = 1'b1;
            end

            op_jmp_ret_set_pc: begin
                mem_read_is_sp = 1'b1;
                pc_data_source = pc_data_source_t::mem;
                set_pc = 1'b1;
            end

            op_store: begin
                mem_write = 1'b1;
                set_pc = 1'b1;
            end
            
            op_push_store: begin
                mem_write = 1'b1;
                mem_write_addr_source = mem_write_addr_source_t::sp;
            end
            
            op_push_inc: begin
                set_sp = 1'b1;
                increase_sp = 1'b1;
                set_pc = 1'b1;
            end
            
            op_jmp_link_inc, op_irq_jmp_link_inc, op_irq_jmp_status_inc: begin
                set_sp = 1'b1;
                increase_sp = 1'b1;
            end
            
            op_pop_dec, op_jmp_ret_dec, op_rti_dec: begin
                set_sp = 1'b1;
                increase_sp = 1'b0;
            end

            op_jmp_link: begin
                mem_write = 1'b1;
                mem_write_addr_source = mem_write_addr_source_t::sp;
                mem_write_data_source = mem_write_data_source_t::next_pc;
            end
            
            op_irq_jmp_status: begin
                mem_write = 1'b1;
                mem_write_addr_source = mem_write_addr_source_t::sp;
                mem_write_data_source = mem_write_data_source_t::sr;
            end

            op_irq_jmp_link: begin
                mem_write = 1'b1;
                mem_write_addr_source = mem_write_addr_source_t::sp;
                mem_write_data_source = mem_write_data_source_t::this_pc;
            end
            
            op_jmp: begin
                pc_data_source = do_jump ? pc_data_source_t::register : pc_data_source_t::next_pc;
                set_pc = 1'b1;
            end
            
            op_irq_jmp: begin
                pc_data_source = pc_data_source_t::irq;
                set_pc = 1'b1;
            end
            
            op_irq_reset: begin
                reset_irq = 1'b1;
            end
            
            op_alu: begin
                reg_write = aluop != 4'b0111;
                alu_set_flags = 1'b1;
                set_pc = 1'b1;
                alu_override_imm4 = opcode == 4'b1001;
            end

            op_imov: begin
                reg_write = 1'b1;
                set_pc = 1'b1;
                alu_output_override = opcode == 4'b0010 ? alu_output_override_t::imm8 : alu_output_override_t::imm8_high;
            end
            
            op_halt:;
        endcase
    end
    
    cpu_state op_decode_state;
    always_comb begin: op_logic
        case(opcode)
            4'b0000: op_decode_state = op_load_set_addr;
            4'b0001: op_decode_state = op_store;
            4'b0101: op_decode_state = op_push_store;
            4'b0110: op_decode_state = op_pop_dec;
            4'b0010, 4'b0011: op_decode_state = op_imov;
            4'b1010: op_decode_state = (ret_jump & do_jump) ? op_jmp_ret_dec : ((link_jump & do_jump) ? op_jmp_link : op_jmp); // if do_jump is false, fallback to op_jmp and inc PC
            4'b1100: op_decode_state = op_rti_dec;
            4'b0111: op_decode_state = op_halt;
            4'b1000, 4'b1001: op_decode_state = op_alu;
            default: op_decode_state = op_halt;
        endcase
    end
    
    always_comb begin: next_state_logic
        unique case(curr_state)
            reset_state					: next_state = irq ? op_irq_jmp_link : fetch_set_addr;
            fetch_set_addr				: next_state = fetch_set_instruction;
            fetch_set_instruction		: next_state = op_decode;
            
            op_decode					: next_state = op_decode_state;
            
            op_load_set_addr			: next_state = op_load_set_register;
            op_load_set_register		: next_state = reset_state;
            
            op_store					: next_state = reset_state;
            
            op_push_store				: next_state = op_push_inc;
            op_push_inc					: next_state = reset_state;
            
            op_pop_dec					: next_state = op_pop_set_addr;
            op_pop_set_addr				: next_state = op_pop_set_register;
            op_pop_set_register			: next_state = reset_state;
            
            op_jmp_link					: next_state = op_jmp_link_inc;
            op_jmp_link_inc				: next_state = op_jmp;
            op_jmp						: next_state = reset_state;
            
            op_jmp_ret_dec				: next_state = op_jmp_ret_set_addr;
            op_jmp_ret_set_addr			: next_state = op_jmp_ret_set_pc;
            op_jmp_ret_set_pc			: next_state = reset_state;
                
            op_irq_jmp_link				: next_state = op_irq_jmp_link_inc;
            op_irq_jmp_link_inc			: next_state = op_irq_jmp_status;
            op_irq_jmp_status			: next_state = op_irq_jmp_status_inc;
            op_irq_jmp_status_inc		: next_state = op_irq_jmp;
            op_irq_jmp					: next_state = op_irq_reset;
            op_irq_reset				: next_state = reset_state;
            
            op_rti_dec					: next_state = op_rti_set_addr;
            op_rti_set_addr				: next_state = op_rti_set_sr;
            op_rti_set_sr				: next_state = op_jmp_ret_dec;
            
            op_alu						: next_state = reset_state;

            op_imov                     : next_state = reset_state;
            
            op_halt						: next_state = op_halt;				// halt
        endcase
    end

endmodule