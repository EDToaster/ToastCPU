module main(
	input CLOCK_50,
	input [3:0] KEY,
	input [9:0] SW,
	
	output [9:0] LEDR,
	output [6:0] HEX0,
	output [6:0] HEX1,
	output [6:0] HEX2,
	output [6:0] HEX3,
	output [6:0] HEX4,
	output [6:0] HEX5
);

	wire reset = KEY[0];

	// create slower clock
	reg [23:0] counter;
	wire	 slow_clock = counter[1];
	//wire slow_clock = ~KEY[1];
	
	always_ff @(posedge CLOCK_50 or negedge reset)
	begin: clock_divide
		if (~reset) begin
			counter <= 24'd0;
		end else begin
			counter = counter - 1'b1;
		end
	
	end
	
	logic [15:0] pc, mem, instruction;
	logic reg_write, mem_to_reg, fetch_instruction, alu_override_imm, alu_override_b, 
			alu_set_flags, set_pc, pc_from_register, mem_write;
//	logic do_halt;
	logic Z;
	
	logic [15:0] register_datapoke;
	
	datapath (
		.clock(slow_clock),
		.reset,					// reset low
		
		// control signals
		.reg_write,
		.mem_to_reg, 			// transfer memory to reg? (for load, etc)
		.fetch_instruction,	// on clock
		.alu_override_imm,		// override output of alu to be imm16 value?
		.alu_override_b,		// override input b of alu to be 1?
		.alu_set_flags,			// on clock, set status flags?
		.set_pc,					// on clock
		.pc_from_register,		
		.mem_write,
		
		//.do_halt,
		.Z_out(Z),
		.current_instruction(instruction),
		.PC_poke(pc),
		.mem_poke(mem),
		
		.register_addrpoke(SW[3:0]),
		.register_datapoke
	);
	
	controlpath (
		.clock(slow_clock),
		.reset, // reset low
		//.do_halt,
		.Z,
	
		.instruction,

	
		.reg_write,
		.mem_to_reg, 			// transfer memory to reg? (for load, etc)
		.fetch_instruction,	// on clock
		.alu_override_imm,		// override output of alu to be imm16 value?
		.alu_override_b,		// override input b of alu to be 1?
		.alu_set_flags,			// on clock, set status flags?
		.set_pc,					// on clock
		.pc_from_register,		
		.mem_write,
		.state(LEDR[9:0])
	);
	
	display_word d_register(
		register_datapoke,
		HEX0, HEX1, HEX2, HEX3
	);
	
	display_byte d_pc(
		pc[7:0],
		HEX4, HEX5
	);
	
	//assign LEDR[9] = fetch_instruction;
	//assign LEDR[8] = Z;
	
	
endmodule