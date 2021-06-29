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

	
	always_ff @(posedge io.clock) begin: set_letter
		if (io.wenable) begin
			text_buffer[io.waddr[11:0]] <= io.wdata[7:0];
			text_background[io.waddr[11:0]] <= io.wdata[10:8];
			text_foreground[io.waddr[11:0]] <= io.wdata[13:11];
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
	
	// memory control (ascii, col x row)
	// 64 col, 32 row
	// 8x8 letters
	logic [7:0] text_buffer[0:64*40-1];
	logic [2:0] text_background[0:64*40-1];
	logic [2:0] text_foreground[0:64*40-1];
	
	logic is_foreground;
	logic [2:0] foreground, background, colour;
	
	logic [6:0] text_x;
	logic [6:0] text_y;
	logic [3:0] bitmap_x;
	logic [3:0] bitmap_y;
	
	logic [7:0] ascii;
	logic [0:8*12-1] bitmap;
	
	logic [11:0] x, y;
	always_ff @(posedge VGA_CLK or negedge reset) begin: pixel_counters
		if (~reset) begin
			x <= 11'b0;
			y <= 11'b0;
			text_x <= 7'b0;
			text_y <= 6'b0;
		end else begin
			if (x < (h_ap + h_fp + h_sw + h_bp - 1)) begin
				// next pixel
				x <= x + 1'b1;
				
				if (bitmap_x == 4'h7) begin
					bitmap_x <= 4'b0;
					text_x <= text_x + 1'b1;
				end else begin
					bitmap_x <= bitmap_x + 1'b1;
				end
				
			end else begin
				// next line
				x <= 11'b0;
				text_x <= 7'b0;
				bitmap_x <= 4'b0;
				
				if (y < (v_al + v_fp + v_sw + v_bp - 1)) begin
					y <= y + 1'b1;
					
					if (bitmap_y == 4'd11) begin
						bitmap_y <= 4'b0;
						text_y <= text_y + 1'b1;
					end else begin
						bitmap_y <= bitmap_y + 1'b1;
					end
				end else begin
					y <= 11'b0;
					text_y <= 6'b0;
					bitmap_y <= 4'b0;
				end
			end
			
			text_valid = /*~y[3] &&*/ text_x < 64 && text_y < 40;	// every other one
			
			ascii = text_valid ? text_buffer[{text_y[5:0], text_x[5:0]}] : 8'h0;
			foreground = text_valid ? text_foreground[{text_y[5:0], text_x[5:0]}] : 3'b0;
			background = text_valid ? text_background[{text_y[5:0], text_x[5:0]}] : 3'b0;
			
			// lut needs to be synchronous here because when it wasn't it was very scuffed.
			unique case(ascii) 
				8'h00 : bitmap = 96'h000000000000000000000000;
				8'h01 : bitmap = 96'h007EC381A581BD99C37E0000;
				8'h02 : bitmap = 96'h007EFFFFDBFFC3E7FF7E0000;
				8'h03 : bitmap = 96'h000044EEFEFEFE7C38100000;
				8'h04 : bitmap = 96'h0010387CFEFE7C3810000000;
				8'h05 : bitmap = 96'h00183C3CFFE7E718187E0000;
				8'h06 : bitmap = 96'h00183C7EFFFF7E18187E0000;
				8'h07 : bitmap = 96'h000000003C7E7E3C00000000;
				8'h08 : bitmap = 96'hFFFFFFFFC38181C3FFFFFFFF;
				8'h09 : bitmap = 96'h00003C7E664242667E3C0000;
				8'h0A : bitmap = 96'hFFFFC38199BDBD9981C3FFFF;
				8'h0B : bitmap = 96'h003E0E3A72F8CCCCCC780000;
				8'h0C : bitmap = 96'h003C6666663C187E18180000;
				8'h0D : bitmap = 96'h001F19191F181878F8700000;
				8'h0E : bitmap = 96'h007F637F63636367E7E6C000;
				8'h0F : bitmap = 96'h000018DB7EE7E77EDB180000;
				8'h10 : bitmap = 96'h0080C0E0F8FEF8E0C0800000;
				8'h11 : bitmap = 96'h0002060E3EFE3E0E06020000;
				8'h12 : bitmap = 96'h00183C7E1818187E3C180000;
				8'h13 : bitmap = 96'h006666666666000066660000;
				8'h14 : bitmap = 96'h007FDBDBDB7B1B1B1B1B0000;
				8'h15 : bitmap = 96'h007E63303C66663C0CC67E00;
				8'h16 : bitmap = 96'h00000000000000FEFEFE0000;
				8'h17 : bitmap = 96'h00183C7E1818187E3C187E00;
				8'h18 : bitmap = 96'h00183C7E1818181818180000;
				8'h19 : bitmap = 96'h001818181818187E3C180000;
				8'h1A : bitmap = 96'h000000180CFE0C1800000000;
				8'h1B : bitmap = 96'h0000003060FE603000000000;
				8'h1C : bitmap = 96'h00000000C0C0C0FE00000000;
				8'h1D : bitmap = 96'h0000002466FF662400000000;
				8'h1E : bitmap = 96'h0000101038387C7CFEFE0000;
				8'h1F : bitmap = 96'h0000FEFE7C7C383810100000;
				8'h20 : bitmap = 96'h000000000000000000000000;
				8'h21 : bitmap = 96'h003078787830300030300000;
				8'h22 : bitmap = 96'h006666662400000000000000;
				8'h23 : bitmap = 96'h006C6CFE6C6C6CFE6C6C0000;
				8'h24 : bitmap = 96'h30307CC0C0780C0CF8303000;
				8'h25 : bitmap = 96'h000000C4CC183060CC8C0000;
				8'h26 : bitmap = 96'h0070D8D870FADECCDC760000;
				8'h27 : bitmap = 96'h003030306000000000000000;
				8'h28 : bitmap = 96'h000C183060606030180C0000;
				8'h29 : bitmap = 96'h006030180C0C0C1830600000;
				8'h2A : bitmap = 96'h000000663CFF3C6600000000;
				8'h2B : bitmap = 96'h00000018187E181800000000;
				8'h2C : bitmap = 96'h000000000000000038386000;
				8'h2D : bitmap = 96'h0000000000FE000000000000;
				8'h2E : bitmap = 96'h000000000000000038380000;
				8'h2F : bitmap = 96'h000002060C183060C0800000;
				8'h30 : bitmap = 96'h007CC6CEDED6F6E6C67C0000;
				8'h31 : bitmap = 96'h001030F03030303030FC0000;
				8'h32 : bitmap = 96'h0078CCCC0C183060CCFC0000;
				8'h33 : bitmap = 96'h0078CC0C0C380C0CCC780000;
				8'h34 : bitmap = 96'h000C1C3C6CCCFE0C0C1E0000;
				8'h35 : bitmap = 96'h00FCC0C0C0F80C0CCC780000;
				8'h36 : bitmap = 96'h003860C0C0F8CCCCCC780000;
				8'h37 : bitmap = 96'h00FEC6C6060C183030300000;
				8'h38 : bitmap = 96'h0078CCCCEC78DCCCCC780000;
				8'h39 : bitmap = 96'h0078CCCCCC7C181830700000;
				8'h3A : bitmap = 96'h000000383800003838000000;
				8'h3B : bitmap = 96'h000000383800003838183000;
				8'h3C : bitmap = 96'h000C183060C06030180C0000;
				8'h3D : bitmap = 96'h000000007E007E0000000000;
				8'h3E : bitmap = 96'h006030180C060C1830600000;
				8'h3F : bitmap = 96'h0078CC0C1830300030300000;
				8'h40 : bitmap = 96'h007CC6C6DEDEDEC0C07C0000;
				8'h41 : bitmap = 96'h003078CCCCCCFCCCCCCC0000;
				8'h42 : bitmap = 96'h00FC6666667C666666FC0000;
				8'h43 : bitmap = 96'h003C66C6C0C0C0C6663C0000;
				8'h44 : bitmap = 96'h00F86C66666666666CF80000;
				8'h45 : bitmap = 96'h00FE6260647C646062FE0000;
				8'h46 : bitmap = 96'h00FE6662647C646060F00000;
				8'h47 : bitmap = 96'h003C66C6C0C0CEC6663E0000;
				8'h48 : bitmap = 96'h00CCCCCCCCFCCCCCCCCC0000;
				8'h49 : bitmap = 96'h007830303030303030780000;
				8'h4A : bitmap = 96'h001E0C0C0C0CCCCCCC780000;
				8'h4B : bitmap = 96'h00E6666C6C786C6C66E60000;
				8'h4C : bitmap = 96'h00F060606060626666FE0000;
				8'h4D : bitmap = 96'h00C6EEFEFED6C6C6C6C60000;
				8'h4E : bitmap = 96'h00C6C6E6F6FEDECEC6C60000;
				8'h4F : bitmap = 96'h00386CC6C6C6C6C66C380000;
				8'h50 : bitmap = 96'h00FC6666667C606060F00000;
				8'h51 : bitmap = 96'h00386CC6C6C6CEDE7C0C1E00;
				8'h52 : bitmap = 96'h00FC6666667C6C6666E60000;
				8'h53 : bitmap = 96'h0078CCCCC07018CCCC780000;
				8'h54 : bitmap = 96'h00FCB4303030303030780000;
				8'h55 : bitmap = 96'h00CCCCCCCCCCCCCCCC780000;
				8'h56 : bitmap = 96'h00CCCCCCCCCCCCCC78300000;
				8'h57 : bitmap = 96'h00C6C6C6C6D6D66C6C6C0000;
				8'h58 : bitmap = 96'h00CCCCCC783078CCCCCC0000;
				8'h59 : bitmap = 96'h00CCCCCCCC78303030780000;
				8'h5A : bitmap = 96'h00FECE9818306062C6FE0000;
				8'h5B : bitmap = 96'h003C303030303030303C0000;
				8'h5C : bitmap = 96'h000080C06030180C06020000;
				8'h5D : bitmap = 96'h003C0C0C0C0C0C0C0C3C0000;
				8'h5E : bitmap = 96'h10386CC60000000000000000;
				8'h5F : bitmap = 96'h00000000000000000000FF00;
				8'h60 : bitmap = 96'h303018000000000000000000;
				8'h61 : bitmap = 96'h00000000780C7CCCCC760000;
				8'h62 : bitmap = 96'h00E060607C66666666DC0000;
				8'h63 : bitmap = 96'h0000000078CCC0C0CC780000;
				8'h64 : bitmap = 96'h001C0C0C7CCCCCCCCC760000;
				8'h65 : bitmap = 96'h0000000078CCFCC0CC780000;
				8'h66 : bitmap = 96'h00386C6060F8606060F00000;
				8'h67 : bitmap = 96'h0000000076CCCCCC7C0CCC78;
				8'h68 : bitmap = 96'h00E060606C76666666E60000;
				8'h69 : bitmap = 96'h0018180078181818187E0000;
				8'h6A : bitmap = 96'h000C0C003C0C0C0C0CCCCC78;
				8'h6B : bitmap = 96'h00E06060666C786C66E60000;
				8'h6C : bitmap = 96'h0078181818181818187E0000;
				8'h6D : bitmap = 96'h00000000FCD6D6D6D6C60000;
				8'h6E : bitmap = 96'h00000000F8CCCCCCCCCC0000;
				8'h6F : bitmap = 96'h0000000078CCCCCCCC780000;
				8'h70 : bitmap = 96'h00000000DC666666667C60F0;
				8'h71 : bitmap = 96'h0000000076CCCCCCCC7C0C1E;
				8'h72 : bitmap = 96'h00000000EC6E766060F00000;
				8'h73 : bitmap = 96'h0000000078CC6018CC780000;
				8'h74 : bitmap = 96'h00002060FC6060606C380000;
				8'h75 : bitmap = 96'h00000000CCCCCCCCCC760000;
				8'h76 : bitmap = 96'h00000000CCCCCCCC78300000;
				8'h77 : bitmap = 96'h00000000C6C6D6D66C6C0000;
				8'h78 : bitmap = 96'h00000000C66C38386CC60000;
				8'h79 : bitmap = 96'h00000000666666663C0C18F0;
				8'h7A : bitmap = 96'h00000000FC8C1860C4FC0000;
				8'h7B : bitmap = 96'h001C303060C06030301C0000;
				8'h7C : bitmap = 96'h001818181800181818180000;
				8'h7D : bitmap = 96'h00E03030180C183030E00000;
				8'h7E : bitmap = 96'h0073DACE0000000000000000;
				8'h7F : bitmap = 96'h00000010386CC6C6FE000000;
				8'h80 : bitmap = 96'h0078CCCCC0C0C0CCCC7830F0;
				8'h81 : bitmap = 96'h00CCCC00CCCCCCCCCC760000;
				8'h82 : bitmap = 96'h0C18300078CCFCC0CC780000;
				8'h83 : bitmap = 96'h3078CC00780C7CCCCC760000;
				8'h84 : bitmap = 96'h00CCCC00780C7CCCCC760000;
				8'h85 : bitmap = 96'hC0603000780C7CCCCC760000;
				8'h86 : bitmap = 96'h386C6C38F80C7CCCCC760000;
				8'h87 : bitmap = 96'h0000000078CCC0C0CC7830F0;
				8'h88 : bitmap = 96'h3078CC0078CCFCC0C07C0000;
				8'h89 : bitmap = 96'h00CCCC0078CCFCC0C07C0000;
				8'h8A : bitmap = 96'hC060300078CCFCC0C07C0000;
				8'h8B : bitmap = 96'h006C6C0078181818187E0000;
				8'h8C : bitmap = 96'h10386C0078181818187E0000;
				8'h8D : bitmap = 96'h6030180078181818187E0000;
				8'h8E : bitmap = 96'h00CC003078CCCCFCCCCC0000;
				8'h8F : bitmap = 96'h78CCCC7878CCCCFCCCCC0000;
				8'h90 : bitmap = 96'h0C1830FCC4C0F8C0C4FC0000;
				8'h91 : bitmap = 96'h00000000FE1B7FD8D8EF0000;
				8'h92 : bitmap = 96'h003E78D8D8FED8D8D8DE0000;
				8'h93 : bitmap = 96'h3078CC0078CCCCCCCC780000;
				8'h94 : bitmap = 96'h00CCCC0078CCCCCCCC780000;
				8'h95 : bitmap = 96'hC060300078CCCCCCCC780000;
				8'h96 : bitmap = 96'h3078CC00CCCCCCCCCC760000;
				8'h97 : bitmap = 96'hC0603000CCCCCCCCCC760000;
				8'h98 : bitmap = 96'h00666600666666663C0C18F0;
				8'h99 : bitmap = 96'hCC0078CCCCCCCCCCCC780000;
				8'h9A : bitmap = 96'hCC00CCCCCCCCCCCCCC780000;
				8'h9B : bitmap = 96'h00303078CCC0C0CC78303000;
				8'h9C : bitmap = 96'h3C66606060FC6060C0FE0000;
				8'h9D : bitmap = 96'hCCCCCCCC78FC30FC30300000;
				8'h9E : bitmap = 96'hF0888888F0889E8C8D860000;
				8'h9F : bitmap = 96'h0E1B18187E181818D8700000;
				8'hA0 : bitmap = 96'h0C183000780C7CCCCC760000;
				8'hA1 : bitmap = 96'h0C18300078181818187E0000;
				8'hA2 : bitmap = 96'h0C18300078CCCCCCCC780000;
				8'hA3 : bitmap = 96'h0C183000CCCCCCCCCC760000;
				8'hA4 : bitmap = 96'h0076DC00F8CCCCCCCCCC0000;
				8'hA5 : bitmap = 96'h76DC00C6E6F6DECEC6C60000;
				8'hA6 : bitmap = 96'h0078CCCC7E00FE0000000000;
				8'hA7 : bitmap = 96'h0078CCCC7800FE0000000000;
				8'hA8 : bitmap = 96'h003030003060C0C0CC780000;
				8'hA9 : bitmap = 96'h0000000000FCC0C0C0000000;
				8'hAA : bitmap = 96'h0000000000FC0C0C0C000000;
				8'hAB : bitmap = 96'h0042C6CCD8306EC3860C1F00;
				8'hAC : bitmap = 96'h0063E66C78376FDBB33F0300;
				8'hAD : bitmap = 96'h003030003030787878300000;
				8'hAE : bitmap = 96'h000000003366CCCC66330000;
				8'hAF : bitmap = 96'h00000000CC66333366CC0000;
				8'hB0 : bitmap = 96'h249249249249249249249249;
				8'hB1 : bitmap = 96'h55AA55AA55AA55AA55AA55AA;
				8'hB2 : bitmap = 96'h6DDBB66DDBB66DDBB66DDBB6;
				8'hB3 : bitmap = 96'h181818181818181818181818;
				8'hB4 : bitmap = 96'h1818183060C0603018181818;
				8'hB5 : bitmap = 96'h18183060C00000C060301818;
				8'hB6 : bitmap = 96'h6666666666C6666666666666;
				8'hB7 : bitmap = 96'h0000000000FC6E6666666666;
				8'hB8 : bitmap = 96'h00000000F01808E838181818;
				8'hB9 : bitmap = 96'h66666666C60606C666666666;
				8'hBA : bitmap = 96'h666666666666666666666666;
				8'hBB : bitmap = 96'h00000000E03018CC66666666;
				8'hBC : bitmap = 96'h66666666CC1830E000000000;
				8'hBD : bitmap = 96'h666666666EFC000000000000;
				8'hBE : bitmap = 96'h18181838E80818F000000000;
				8'hBF : bitmap = 96'h0000000000C0603018181818;
				8'hC0 : bitmap = 96'h1818180C0603000000000000;
				8'hC1 : bitmap = 96'h1818183C66C3000000000000;
				8'hC2 : bitmap = 96'h0000000000C3663C18181818;
				8'hC3 : bitmap = 96'h1818180C0603060C18181818;
				8'hC4 : bitmap = 96'h0000000000FF000000000000;
				8'hC5 : bitmap = 96'h1818183C66C3663C18181818;
				8'hC6 : bitmap = 96'h18180C0603000003060C1818;
				8'hC7 : bitmap = 96'h666666666663666666666666;
				8'hC8 : bitmap = 96'h6666666633180C0700000000;
				8'hC9 : bitmap = 96'h00000000070C183366666666;
				8'hCA : bitmap = 96'h66666666C30000FF00000000;
				8'hCB : bitmap = 96'h00000000FF0000C366666666;
				8'hCC : bitmap = 96'h666666666360606366666666;
				8'hCD : bitmap = 96'h00000000FF0000FF00000000;
				8'hCE : bitmap = 96'h66666666C31818C366666666;
				8'hCF : bitmap = 96'h18183C66C30000FF00000000;
				8'hD0 : bitmap = 96'h6666666666FF000000000000;
				8'hD1 : bitmap = 96'h00000000FF0000FF18181818;
				8'hD2 : bitmap = 96'h0000000000FF666666666666;
				8'hD3 : bitmap = 96'h66666666763F000000000000;
				8'hD4 : bitmap = 96'h1818181C1710180F00000000;
				8'hD5 : bitmap = 96'h000000000F1810171C181818;
				8'hD6 : bitmap = 96'h00000000003F766666666666;
				8'hD7 : bitmap = 96'h6666666666C3666666666666;
				8'hD8 : bitmap = 96'h18183C66C31818C3663C1818;
				8'hD9 : bitmap = 96'h1818183060C0000000000000;
				8'hDA : bitmap = 96'h000000000003060C18181818;
				8'hDB : bitmap = 96'hFFFFFFFFFFFFFFFFFFFFFFFF;
				8'hDC : bitmap = 96'h000000000000FFFFFFFFFFFF;
				8'hDD : bitmap = 96'hF0F0F0F0F0F0F0F0F0F0F0F0;
				8'hDE : bitmap = 96'h0F0F0F0F0F0F0F0F0F0F0F0F;
				8'hDF : bitmap = 96'hFFFFFFFFFFFF000000000000;
				8'hE0 : bitmap = 96'h0000000076DECCCCDE760000;
				8'hE1 : bitmap = 96'h0078CCCCD8CCCCCCF8C06000;
				8'hE2 : bitmap = 96'h00FCCCCCC0C0C0C0C0C00000;
				8'hE3 : bitmap = 96'h00FE6C6C6C6C6C6C6C660000;
				8'hE4 : bitmap = 96'h00FCC46460306064C4FC0000;
				8'hE5 : bitmap = 96'h000000007EC8CCCCCC780000;
				8'hE6 : bitmap = 96'h0000000066666666667B60C0;
				8'hE7 : bitmap = 96'h00000076DC181818180E0000;
				8'hE8 : bitmap = 96'h00FC3078CCCCCC7830FC0000;
				8'hE9 : bitmap = 96'h0078CCCCCCFCCCCCCC780000;
				8'hEA : bitmap = 96'h007CC6C6C6C66C6C6CEE0000;
				8'hEB : bitmap = 96'h003C603078CCCCCCCC780000;
				8'hEC : bitmap = 96'h00000076DBDBDB6E00000000;
				8'hED : bitmap = 96'h0000067CDED6F67CC0000000;
				8'hEE : bitmap = 96'h003C60C0C0FCC0C0603C0000;
				8'hEF : bitmap = 96'h000078CCCCCCCCCCCCCC0000;
				8'hF0 : bitmap = 96'h0000FC0000FC0000FC000000;
				8'hF1 : bitmap = 96'h00003030FC303000FC000000;
				8'hF2 : bitmap = 96'h0060301818306000FC000000;
				8'hF3 : bitmap = 96'h0018306060301800FC000000;
				8'hF4 : bitmap = 96'h00000E1B1B18181818181818;
				8'hF5 : bitmap = 96'h18181818181818D8D8700000;
				8'hF6 : bitmap = 96'h0000303000FC003030000000;
				8'hF7 : bitmap = 96'h000073DBCE0073DBCE000000;
				8'hF8 : bitmap = 96'h003C6666663C000000000000;
				8'hF9 : bitmap = 96'h000000001C1C000000000000;
				8'hFA : bitmap = 96'h000000000018000000000000;
				8'hFB : bitmap = 96'h00070404044464341C0C0000;
				8'hFC : bitmap = 96'h00D86C6C6C6C000000000000;
				8'hFD : bitmap = 96'h00780C18307C000000000000;
				8'hFE : bitmap = 96'h00003C3C3C3C3C3C3C3C0000;
				8'hFF : bitmap = 96'h000000000000000000000000;
			endcase
			
			is_foreground = bitmap[{bitmap_y[3:0], bitmap_x[2:0]}];
			colour = is_foreground ? foreground : background;
		end
	end
	
	
	
	
	
	
	
	
	
	
	
	

endmodule