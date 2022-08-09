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
	output logic fetch_instruction,	// on clock
	output logic alu_override_imm8,	// override output of alu to be imm16 value?
	output logic alu_override_imm4,		// override input b of alu to be imm4 value?
	output logic alu_set_flags,		// on clock, set status flags?
	output logic set_pc,					// on clock
	output logic pc_from_irq,
	output logic pc_from_register,	
	output logic reset_irq,				// todo: posedge clock reset IRQ line (currently async)
	
	output logic set_sp,
	output logic increase_sp,
	
	output logic mem_write,
	output logic mem_write_is_stack,	// for push, jal instructions
	output logic mem_write_next_pc,	// for jal instructions
	output logic mem_write_this_pc,  // for irq, since we want to store return location on stack
		
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
		
		op_jmp_link,
		op_jmp_link_inc,
		op_jmp,
		
		// irq
		op_irq_jmp_link,
		op_irq_jmp_link_inc,
		op_irq_jmp,
		op_irq_reset,
		
		op_alu,
		
		op_halt 					// halt
	} cpu_state;
	
	cpu_state curr_state, next_state;
	
	wire [3:0] opcode = instruction[15:12];
	wire [3:0] jop = instruction[3:0];
	wire override_b = instruction[3];
	wire link_jump = instruction[4];
	
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
		fetch_instruction = 1'b0;
		alu_override_imm8 = 1'b0;
		alu_override_imm4 = 1'b0;
		alu_set_flags = 1'b0;
		set_pc = 1'b0;
		pc_from_register = 1'b0;
		pc_from_irq = 1'b0;
		mem_write = 1'b0;
		mem_write_is_stack = 1'b0;
		mem_write_next_pc = 1'b0;
		mem_write_this_pc = 1'b0;
		set_sp = 1'b0;
		increase_sp = 1'b0;
		reset_irq = 1'b0;
		
		unique case(curr_state)
			reset_state:;
			
			fetch_set_addr,
			fetch_set_instruction: fetch_instruction = 1'b1;
			
			op_decode:;
			
			op_load_set_addr, op_pop_set_addr:;
			op_load_set_register, op_pop_set_register: begin
				reg_write = 1'b1;
				mem_to_reg = 1'b1;
				set_pc = 1'b1;
			end
			
			op_store: begin
				mem_write = 1'b1;
				set_pc = 1'b1;
			end
			
			op_push_store: begin
				mem_write = 1'b1;
				mem_write_is_stack = 1'b1;
			end
			
			op_push_inc, op_jmp_link_inc, op_irq_jmp_link_inc: begin
				set_sp = 1'b1;
				increase_sp = 1'b1;
			end
			
			op_pop_dec: begin
				set_sp = 1'b1;
				increase_sp = 1'b0;
			end
			
			op_jmp_link: begin
				mem_write = 1'b1;
				mem_write_is_stack = 1'b1;
				mem_write_next_pc = 1'b1;
			end
			
			op_irq_jmp_link: begin
				mem_write = 1'b1;
				mem_write_is_stack = 1'b1;
				mem_write_this_pc = 1'b1;
			end
			
			op_jmp: begin
				pc_from_register = do_jump;
				set_pc = 1'b1;
			end
			
			op_irq_jmp: begin
				pc_from_register = 1'b1;
				pc_from_irq = 1'b1;
				set_pc = 1'b1;
			end
			
			op_irq_reset: begin
				reset_irq = 1'b1;
			end
			
			op_alu: begin
				reg_write = 1'b1;
				alu_set_flags = 1'b1;
				set_pc = 1'b1;
				alu_override_imm8 = opcode == 4'b0010;
				alu_override_imm4 = opcode == 4'b1001;
			end
			
			op_halt:; 					// halt
		endcase
	end
	
	cpu_state op_decode_state;
	always_comb begin: op_logic
		case(opcode)
			4'b0000: op_decode_state = op_load_set_addr;
			4'b0001: op_decode_state = op_store;
			4'b0101: op_decode_state = op_push_store;
			4'b0110: op_decode_state = op_pop_dec;
			4'b0010: op_decode_state = op_alu;
			4'b1010: op_decode_state = (link_jump & do_jump) ? op_jmp_link : op_jmp;
			4'b0111: op_decode_state = op_halt;
			4'b1000, 4'b1001: op_decode_state = op_alu;
			default: op_decode_state = op_halt;
		endcase
	end
	
	always_comb begin: next_state_logic
		unique case(curr_state)
			reset_state: next_state = irq ? op_irq_jmp_link : fetch_set_addr;
			fetch_set_addr: next_state = fetch_set_instruction;
			fetch_set_instruction: next_state = op_decode;
			
			op_decode: next_state = op_decode_state;
			
			op_load_set_addr: next_state = op_load_set_register;
			op_load_set_register: next_state = reset_state;
			
			op_store: next_state = reset_state;
			
			op_push_store: next_state = op_push_inc;
			op_push_inc: next_state = reset_state;
			
			op_pop_dec: next_state = op_pop_set_addr;
			op_pop_set_addr: next_state = op_pop_set_register;
			op_pop_set_register: next_state = reset_state;
			
			op_jmp_link: next_state = op_jmp_link_inc;
			op_jmp_link_inc: next_state = op_jmp;
			op_jmp: next_state = reset_state;
			
			op_irq_jmp_link: next_state = op_irq_jmp_link_inc;
			op_irq_jmp_link_inc: next_state = op_irq_jmp;
			op_irq_jmp: next_state = op_irq_reset;
			op_irq_reset: next_state = reset_state;
			
			op_alu: next_state = reset_state;
			
			op_halt: next_state = op_halt;				// halt
		endcase
	end

endmodule