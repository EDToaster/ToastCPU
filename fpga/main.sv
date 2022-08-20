`include "types/control_signals_imports.svh"

module main(
	input logic CLOCK_50,
	input logic [3:0] KEY,
	input logic [9:0] SW,
	input logic PS2_CLK,
	input logic PS2_DAT,
	
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
	output logic [7:0] VGA_R,   						//	VGA Red[7:0]
	output logic [7:0] VGA_G,	 						//	VGA Green[7:0]
	output logic [7:0] VGA_B   						//	VGA Blue[7:0]
);

	// controls
	// KEY[0] = ~reset
	// SW[9]  = clock speed switch
	// SW[8]  = slow clock increment enable

	wire reset = KEY[0];

	// create slower clock
	wire slow_clock = SW[9] ? (SW[8] ? counter[14] : KEY[1]) : CLOCK_50;
	
	logic [26:0] counter;
	always_ff @(posedge CLOCK_50)
	begin
		counter <= counter + 1'b1;
	end
	
	logic [15:0] pc, mem, instruction;
	logic reg_write, mem_to_reg, mem_read_is_pc, mem_read_is_sp, alu_override_imm8, alu_override_imm4, 
			alu_set_flags, set_pc, pc_from_register, pc_from_irq, pc_from_mem, sr_from_mem, set_sp, increase_sp, mem_write;

    mem_write_addr_source_t::t mem_write_addr_source;
    mem_write_data_source_t::t mem_write_data_source;

	logic Z, N;
	
	logic [15:0] register_datapoke;
	
	// irq
	logic irq;
	logic reset_irq;
	
	datapath (
		.clock(slow_clock),
		.reset,					// reset low
		
		// control signals
		.reg_write,
		.mem_to_reg, 			// transfer memory to reg? (for load, etc)
		.mem_read_is_pc,	// on clock
		.mem_read_is_sp,	// on clock
		.alu_override_imm8,	
		.alu_override_imm4,
		.alu_set_flags,			// on clock, set status flags?
		.set_pc,					// on clock
		.pc_from_register,
		.pc_from_irq,
		.pc_from_mem,
		.sr_from_mem,
		.reset_irq,				// todo: set on clock posedge :(
	
		.set_sp,
		.increase_sp,
	
		.mem_write,
        .mem_write_addr_source,
		.mem_write_data_source,
		
		.key_io,
		.vga_io,
		.irq,
		
		//.do_halt,
		.Z_out(Z),
		.N_out(N),
		.current_instruction(instruction),
		.PC_poke(pc),
		.mem_poke(mem),
		
		.register_addrpoke(SW[3:0]),
		.register_datapoke,
		
		//.HEX0, .HEX1, .HEX2, .HEX3
	);
	
	controlpath (
		.clock(slow_clock),
		.reset, // reset low
		//.do_halt,
		.Z,
		.N,
		.irq,
	
		.instruction,

	
		.reg_write,
		.mem_to_reg, 			// transfer memory to reg? (for load, etc)
		.mem_read_is_pc,	// on clock
		.mem_read_is_sp,	// on clock
		.alu_override_imm8,	
		.alu_override_imm4,	
		.alu_set_flags,			// on clock, set status flags?
		.set_pc,					// on clock
		.pc_from_register,	
		.pc_from_irq,
		.pc_from_mem,
		.sr_from_mem,
		.reset_irq,
		
		.set_sp, 
		.increase_sp,
		
		.mem_write,
        .mem_write_addr_source,
		.mem_write_data_source
		//.state(LEDR[9:0])
	);

	/**
	 * DRIVERS
     */

	io_interface key_io();
	key_driver (
		.clk(PS2_CLK),  // Clock pin form keyboard
		.pd(PS2_DAT), 	//Data pin form keyboard
		.io(key_io)
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

	/**
	 * DEBUG DISPLAY
	 */
	assign LEDR[0] = pc_from_register;
	assign LEDR[1] = alu_set_flags;
	
	assign LEDR[9] = irq;
	assign LEDR[8] = reset_irq;

	display_byte d_pc(
		pc[7:0],
		HEX4, HEX5
	);

	display_word d_register(
		register_datapoke,
		HEX0, HEX1, HEX2, HEX3
	);
endmodule