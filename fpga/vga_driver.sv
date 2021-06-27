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

	// memory control (ascii, col x row)
	// 64 col, 32 row
	// 8x8 letters
	logic [7:0] text_buffer[0:2047];
	logic [2:0] text_background[0:2047];
	logic [2:0] text_foreground[0:2047];
	
	always_ff @(posedge io.clock) begin: set_letter
		if (io.wenable) begin
			text_buffer[io.waddr[10:0]] <= io.wdata[7:0];
			text_background[io.waddr[10:0]] <= io.wdata[10:8];
			text_foreground[io.waddr[10:0]] <= io.wdata[13:11];
		end
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
	assign VGA_R = colour[2] ? 8'hFF : 8'h00;
	assign VGA_G = colour[1] ? 8'hFF : 8'h00;
	assign VGA_B = colour[0] ? 8'hFF : 8'h00;
	
	assign hsync = (x < (h_ap + h_fp)) || (x >= (h_ap + h_fp + h_sw));
	assign vsync = (y < (v_al + v_fp)) || (y >= (v_al + v_fp + v_sw));
	assign valid = (x < h_ap) && (y < v_al);
	
	
	// get char position, offset, colour
	logic text_valid;
	
	logic is_foreground;
	logic [2:0] foreground, background, colour;
	
	logic [6:0] text_x;
	logic [5:0] text_y;
	logic [7:0] ascii;
	logic [0:63] bitmap;
	
	logic [11:0] x, y;
	always_ff @(posedge VGA_CLK or negedge reset) begin: pixel_counters
		if (~reset) begin
			x <= 11'b0;
			y <= 11'b0;
		end else begin
			if (x < (h_ap + h_fp + h_sw + h_bp - 1)) begin
				x = x + 1'b1;
			end else begin
				x = 11'b0;
				if (y < (v_al + v_fp + v_sw + v_bp - 1)) begin
					y = y + 1'b1;
				end else begin
					y = 11'b0;
				end
			end 
			
			text_x = x[9:3];
			text_y = y[9:4];
			text_valid = ~y[3] && text_x < 64 && text_y < 32;	// every other one
			
			ascii = text_valid ? text_buffer[{text_y[4:0], text_x[5:0]}] : 8'h0;
			foreground = text_valid ? text_foreground[{text_y[4:0], text_x[5:0]}] : 3'b0;
			background = text_valid ? text_background[{text_y[4:0], text_x[5:0]}] : 3'b0;
			
			// lut needs to be synchronous here because when it wasn't it was very scuffed.
			case(ascii) 
				8'h21 : bitmap = 64'h0040404040004000;
				8'h2C : bitmap = 64'h0000000000606020;
				8'h2E : bitmap = 64'h0000000000606000;
				8'h30 : bitmap = 64'h0018242C34241800;
				8'h31 : bitmap = 64'h0008182808083C00;
				8'h32 : bitmap = 64'h0018240408103C00;
				8'h33 : bitmap = 64'h0018240804241800;
				8'h34 : bitmap = 64'h000818283C080800;
				8'h35 : bitmap = 64'h003C203804043800;
				8'h36 : bitmap = 64'h0018203824241800;
				8'h37 : bitmap = 64'h003C040810101000;
				8'h38 : bitmap = 64'h0018241824241800;
				8'h39 : bitmap = 64'h001824241C041800;
				8'h41 : bitmap = 64'h001824243C242400;
				8'h61 : bitmap = 64'h000018041C241C00;
				8'h42 : bitmap = 64'h0038243824243800;
				8'h62 : bitmap = 64'h0020203824243800;
				8'h43 : bitmap = 64'h0018242020241800;
				8'h63 : bitmap = 64'h0000182420241800;
				8'h44 : bitmap = 64'h0038242424243800;
				8'h64 : bitmap = 64'h0004041C24241C00;
				8'h45 : bitmap = 64'h003C203820203C00;
				8'h65 : bitmap = 64'h000018243C201C00;
				8'h46 : bitmap = 64'h003C203820202000;
				8'h66 : bitmap = 64'h000C103810101000;
				8'h47 : bitmap = 64'h001C20202C241C00;
				8'h67 : bitmap = 64'h00001C24241C0438;
				8'h48 : bitmap = 64'h0024243C24242400;
				8'h68 : bitmap = 64'h0020203824242400;
				8'h49 : bitmap = 64'h0038101010103800;
				8'h69 : bitmap = 64'h0000200020202000;
				8'h4A : bitmap = 64'h001C040404241800;
				8'h6A : bitmap = 64'h0000040004042418;
				8'h4B : bitmap = 64'h0024283028242400;
				8'h6B : bitmap = 64'h0020202830282400;
				8'h4C : bitmap = 64'h0020202020203C00;
				8'h6C : bitmap = 64'h0018080808081C00;
				8'h4D : bitmap = 64'h00446C5444444400;
				8'h6D : bitmap = 64'h0000006C54444400;
				8'h4E : bitmap = 64'h002424342C242400;
				8'h6E : bitmap = 64'h0000002834242400;
				8'h4F : bitmap = 64'h0018242424241800;
				8'h6F : bitmap = 64'h0000001824241800;
				8'h50 : bitmap = 64'h0038242438202000;
				8'h70 : bitmap = 64'h0000182424382020;
				8'h51 : bitmap = 64'h0018242424241A00;
				8'h71 : bitmap = 64'h00001824241C0404;
				8'h52 : bitmap = 64'h0038242438242400;
				8'h72 : bitmap = 64'h0000182420202000;
				8'h53 : bitmap = 64'h001C201804241800;
				8'h73 : bitmap = 64'h0000182018041800;
				8'h54 : bitmap = 64'h007C101010101000;
				8'h74 : bitmap = 64'h0010103810141800;
				8'h55 : bitmap = 64'h0024242424243C00;
				8'h75 : bitmap = 64'h0000002424241800;
				8'h56 : bitmap = 64'h0024242424241800;
				8'h76 : bitmap = 64'h0000002424140800;
				8'h57 : bitmap = 64'h00444444546C4400;
				8'h77 : bitmap = 64'h00000044546C4400;
				8'h58 : bitmap = 64'h0044281028444400;
				8'h78 : bitmap = 64'h0000242418242400;
				8'h59 : bitmap = 64'h0044281010101000;
				8'h79 : bitmap = 64'h000024241C042418;
				8'h5A : bitmap = 64'h003C040810203C00;
				8'h7A : bitmap = 64'h0000380810203800;
				default: bitmap = 64'h0;
			endcase
			
			is_foreground = bitmap[{y[2:0], x[2:0]}];
			colour = is_foreground ? foreground : background;
		end
	end
	
	
	
	
	
	
	
	
	
	
	
	

endmodule