module controlpath(
	input logic clock,
	input logic reset, // reset low
	//input logic do_halt,
	input logic Z,
	input logic [15:0] instruction,
	
	output logic reg_write,
	output logic mem_to_reg, 			// transfer memory to reg? (for load, etc)
	output logic fetch_instruction,	// on clock
	output logic alu_override_imm,	// override output of alu to be imm16 value?
	output logic alu_override_b,		// override input b of alu to be 1?
	output logic alu_set_flags,		// on clock, set status flags?
	output logic set_pc,					// on clock
	output logic pc_from_register,		
	output logic mem_write,
	
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
		
		op_jmp,
		op_alu,
		
		op_halt 					// halt
	} cpu_state;
	
	cpu_state curr_state, next_state;
	
	wire [3:0] opcode = instruction[15:12];
	wire override_b = instruction[3];
	wire negate_jmp = instruction[7];
	
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
	
	// set control signals
	always_comb begin: control_signals
		// all control flags by default are 0
		reg_write = 1'b0;
		mem_to_reg = 1'b0;
		fetch_instruction = 1'b0;
		alu_override_imm = 1'b0;
		alu_override_b = 1'b0;
		alu_set_flags = 1'b0;
		set_pc = 1'b0;
		pc_from_register = 1'b0;	
		mem_write = 1'b0;
		
//			reset, 				// reset
//			fetch_clk_in, 		// set pc into rom addr
//			fetch_set_inst, 	// fetch instruction into registers
//			set_load_addr, 	// set load addr into ram
//			execute, 			// execute instruction (q is not clocked)
//			//set_store_addr, 	// set store address into ram/io
//			store, 				// set registers for next cycle
//			halt 					// halt

		unique case(curr_state)
			reset_state:;
			
			fetch_set_addr,
			fetch_set_instruction: fetch_instruction = 1'b1;
			
			op_decode:;
			
			op_load_set_addr:;
			op_load_set_register: begin
				reg_write = 1'b1;
				mem_to_reg = 1'b1;
				set_pc = 1'b1;
			end
			
			op_store: begin
				mem_write = 1'b1;
				set_pc = 1'b1;
			end
			
			op_jmp: begin
				pc_from_register = Z ^ negate_jmp;
				set_pc = 1'b1;
			end
			
			op_alu: begin
				reg_write = 1'b1;
				alu_set_flags = 1'b1;
				set_pc = 1'b1;
				alu_override_imm = opcode == 4'b0010;
				alu_override_b = override_b;
			end
			
			op_halt:; 					// halt
		endcase
	end
	
	cpu_state op_decode_state;
	always_comb begin: op_logic
		case(opcode)
			4'b0000: op_decode_state = op_load_set_addr;
			4'b0001: op_decode_state = op_store;
			4'b0010: op_decode_state = op_alu;
			4'b0100: op_decode_state = op_jmp;
			4'b0111: op_decode_state = op_halt;
			4'b1000: op_decode_state = op_alu;
			default: op_decode_state = op_halt;
		endcase
	end
	
	always_comb begin: next_state_logic
		unique case(curr_state)
			reset_state: next_state = fetch_set_addr;
			fetch_set_addr: next_state = fetch_set_instruction;
			fetch_set_instruction: next_state = op_decode;
			
			op_decode: next_state = op_decode_state;
			
			op_load_set_addr: next_state = op_load_set_register;
			op_load_set_register: next_state = reset_state;
			
			op_store: next_state = reset_state;
			
			op_jmp: next_state = reset_state;
			op_alu: next_state = reset_state;
			
			op_halt: next_state = op_halt;				// halt
		endcase
	end

endmodule