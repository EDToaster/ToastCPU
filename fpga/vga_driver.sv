module vga_driver(
	input reset,
	input CLOCK_50,
	io_interface io,
	output logic VGA_CLK,   						//	VGA Clock
	output logic VGA_HS,							//	VGA H_SYNC
	output logic VGA_VS,							//	VGA V_SYNC
	output logic VGA_BLANK_N,						//	VGA BLANK
	output logic VGA_SYNC_N,						//	VGA SYNC
	output logic [9:0] VGA_R,   						//	VGA Red[9:0]
	output logic [9:0] VGA_G,	 						//	VGA Green[9:0]
	output logic [9:0] VGA_B   						//	VGA Blue[9:0]
);

	vga_adapter VGA(
		.resetn(reset),
		.clock(io.clock),
		.CLOCK_50,
		.colour(io.wdata[8:0]),
		.x({1'h0, io.waddr[6:0]}),				//8 bits
		.y({1'h0, io.waddr[13:7]}),	// 7 bits
		.plot(io.wenable),
		/* Signals for the DAC to drive the monitor. */
		.VGA_R,
		.VGA_G,
		.VGA_B,
		.VGA_HS,
		.VGA_VS,
		.VGA_BLANK(VGA_BLANK_N),
		.VGA_SYNC(VGA_SYNC_N),
		.VGA_CLK);
	defparam VGA.RESOLUTION = "160x120";
	defparam VGA.MONOCHROME = "FALSE";
	defparam VGA.BITS_PER_COLOUR_CHANNEL = 3;
	defparam VGA.BACKGROUND_IMAGE = "black.mif";
	

endmodule