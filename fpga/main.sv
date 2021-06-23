interface io_interface;
	logic clock;
	
	logic [15:0] waddr, raddr, wdata, rdata;
	logic wenable;
endinterface

module main(
	input logic CLOCK_50,
	input logic [3:0] KEY,
	input logic [9:0] SW,
	
	output logic [9:0] LEDR,
	output logic [6:0] HEX0,
	output logic [6:0] HEX1,
	output logic [6:0] HEX2,
	output logic [6:0] HEX3,
	output logic [6:0] HEX4,
	output logic [6:0] HEX5,
	
	output logic VGA_CLK,   						//	VGA Clock
	output logic VGA_HS,							//	VGA H_SYNC
	output logic VGA_VS,							//	VGA V_SYNC
	output logic VGA_BLANK_N,						//	VGA BLANK
	output logic VGA_SYNC_N,						//	VGA SYNC
	output logic [9:0] VGA_R,   						//	VGA Red[9:0]
	output logic [9:0] VGA_G,	 						//	VGA Green[9:0]
	output logic [9:0] VGA_B   						//	VGA Blue[9:0]
);

	wire reset = KEY[0];

	// create slower clock
	reg [23:0] counter;
	wire	 slow_clock = SW[9] ? ~KEY[1] : CLOCK_50;
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
		
		.hex_io,
		.vga_io,
		
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
		//.state(LEDR[9:0])
	);
	assign LEDR[0] = pc_from_register;
	assign LEDR[1] = alu_set_flags;
	
	display_byte d_pc(
		pc[7:0],
		HEX4, HEX5
	);
	
	//assign LEDR[9] = fetch_instruction;
	//assign LEDR[8] = Z;
	
		// hex driver io
	io_interface hex_io();
//	hex_driver (
//		.io(hex_io),
//		.HEX0, .HEX1, .HEX2, .HEX3
//	);
	display_word(
		register_datapoke,
		HEX0, HEX1, HEX2, HEX3
	);

	
	
	io_interface vga_io();
	vga_driver (	
		.reset,
		.CLOCK_50,
		.io(vga_io),	
		.VGA_R,
		.VGA_G,
		.VGA_B,
		.VGA_HS,
		.VGA_VS,
		.VGA_BLANK_N,
		.VGA_SYNC_N,
		.VGA_CLK
	);	



endmodule