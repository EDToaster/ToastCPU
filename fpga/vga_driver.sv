module _vga_pll (
	clock_in,
	clock_out);

	input	  clock_in;
	output	  clock_out;

	wire [5:0] clock_output_bus;
	wire [1:0] clock_input_bus;
	wire gnd;
	
	assign gnd = 1'b0;
	assign clock_input_bus = { gnd, clock_in }; 

	altpll	altpll_component (
				.inclk (clock_input_bus),
				.clk (clock_output_bus)
				);
	defparam
		altpll_component.operation_mode = "NORMAL",
		altpll_component.intended_device_family = "Cyclone II",
		altpll_component.lpm_type = "altpll",
		altpll_component.pll_type = "FAST",
		/* Specify the input clock to be a 50MHz clock. A 50 MHz clock is present
		 * on PIN_N2 on the DE2 board. We need to specify the input clock frequency
		 * in order to set up the PLL correctly. To do this we must put the input clock
		 * period measured in picoseconds in the inclk0_input_frequency parameter.
		 * 1/(20000 ps) = 0.5 * 10^(5) Hz = 50 * 10^(6) Hz = 50 MHz. */
		altpll_component.inclk0_input_frequency = 20000,
		altpll_component.primary_clock = "INCLK0",
		/* Specify output clock parameters. The output clock should have a
		 * frequency of 25 MHz, with 50% duty cycle. */
		altpll_component.compensate_clock = "CLK0",
		altpll_component.clk0_phase_shift = "0",
		altpll_component.clk0_divide_by = 2,
		altpll_component.clk0_multiply_by = 1,		
		altpll_component.clk0_duty_cycle = 50;
		
	assign clock_out = clock_output_bus[0];

endmodule

module vga_driver(
	input reset,
	input CLOCK_50,
	io_interface io,
	output logic VGA_CLK,   						//	VGA Clock
	output logic VGA_HS,							//	VGA H_SYNC
	output logic VGA_VS,							//	VGA V_SYNC
	output logic VGA_BLANK_N,						//	VGA BLANK
	output logic VGA_SYNC_N,						//	VGA SYNC
	output logic [7:0] VGA_R,   						//	VGA Red[7:0]
	output logic [7:0] VGA_G,	 						//	VGA Green[7:0]
	output logic [7:0] VGA_B   						//	VGA Blue[7:0]
);

//	vga_adapter VGA(
//		.resetn(reset),
//		.clock(io.clock),
//		.CLOCK_50,
//		.colour(io.wdata[14:0]),
//		.x({1'h0, io.waddr[6:0]}),				//8 bits
//		.y(io.waddr[13:7]),	// 7 bits
//		.plot(io.wenable),
//		/* Signals for the DAC to drive the monitor. */
//		.VGA_R,
//		.VGA_G,
//		.VGA_B,
//		.VGA_HS,
//		.VGA_VS,
//		.VGA_BLANK(VGA_BLANK_N),
//		.VGA_SYNC(VGA_SYNC_N),
//		.VGA_CLK);
//	defparam VGA.RESOLUTION = "160x120";
//	defparam VGA.MONOCHROME = "FALSE";
//	defparam VGA.BITS_PER_COLOUR_CHANNEL = 5;
//	defparam VGA.BACKGROUND_IMAGE = "black.mif";

	// memory control (ascii, col x row)
	// 64 col, 32 row
	// 8x8 letters
	logic [7:0] text_buffer[0:63][0:31];
	logic [6:0] init_col, init_row;
	initial begin
		for (init_col = 0; init_col < 64; init_col = init_col + 1) begin
			for (init_row = 0; init_row < 32; init_row = init_row + 1) begin
				text_buffer[init_col][init_row] = init_col + 8'h41;
			end
		end
	end
	
	logic [7:0] ascii;
	logic [7:0][7:0] bitmap;
	
	always_comb begin: bitmap_lut
		case(ascii) 
			8'h00: bitmap <= {8{8'h0}};
			8'h41: bitmap <= { 8'b0, {2{8'b00100100}}, 8'b00111100, {2{8'b00100100}}, 8'b00011000, 8'b0 };
			8'h42: bitmap <= { 8'b0, 8'b000011100, {2{8'b00100100}}, 8'b000011100, 8'b00100100, 8'b00011100 };
			default: bitmap <= {8{8'b01010101}};
		endcase
	end

	// text mode VGA adapter
	_vga_pll(
		.clock_in(CLOCK_50),
		.clock_out(VGA_CLK)
	);
	
	localparam
		h_ap = 640,
		h_fp = 16,
		h_sw = 96,
		h_bp = 48,
		v_al = 480,
		v_fp = 10,
		v_sw = 2,
		v_bp = 33
		
		;
		
	
	
	// SYNC signals
	assign VGA_BLANK_N = valid;
	assign VGA_SYNC_N = 1'b1;
	assign VGA_HS = hsync;
	assign VGA_VS = vsync;
	logic colour;
	assign VGA_R = colour ? 8'hFF : 8'h00;
	assign VGA_G = colour ? 8'hFF : 8'h00;
	assign VGA_B = colour ? 8'hFF : 8'h00;
	
	assign hsync = (x < (h_ap + h_fp)) || (x >= (h_ap + h_fp + h_sw));
	assign vsync = (y < (v_al + v_fp)) || (y >= (v_al + v_fp + v_sw));
	assign valid = (x < h_ap) && (y < v_al);
	
	logic [11:0] x, y;
	
	// get char position, offset, colour
	wire [8:0] x_bufpos = x[11:3];
	wire [8:0] y_bufpos = y[11:3];
	wire [2:0] x_bitpos = x[2:0];
	wire [2:0] y_bitpos = y[2:0];
	assign ascii = text_buffer[x_bufpos][y_bufpos];
	assign colour = bitmap[y_bitpos][x_bitpos];
	
	always_ff @(posedge VGA_CLK or negedge reset) begin: pixel_counters
		if (~reset) begin
			x <= 11'b0;
			y <= 11'b0;
		end else begin
			if (x < (h_ap + h_fp + h_sw + h_bp - 1)) begin
				x <= x + 1'b1;
			end else begin
				x <= 11'b0;
				if (y < (v_al + v_fp + v_sw + v_bp - 1)) begin
					y <= y + 1'b1;
				end else begin
					y <= 11'b0;
				end
			end
		end
	end
	
	
	
	
	
	
	
	
	
	
	
	

endmodule