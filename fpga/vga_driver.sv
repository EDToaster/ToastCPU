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
	localparam 
		bitmap_width = 6,
		bitmap_height = 8,
		bitmap_widthbits = 4,
		bitmap_heightbits = 4,
		bitmap_addressbits = 8,
		
		text_buffer_width = 100,
		text_buffer_height = 60,
		text_buffer_widthbits = 7,
		text_buffer_heightbits = 7,
		text_buffer_addressbits = 14,
		
		vga_offset_bits = 4
		;

	
	always_ff @(posedge io.clock) begin: set_letter
		if (io.wenable) begin
			text_buffer[io.waddr[text_buffer_addressbits-1:0]] <= io.wdata[13:0];
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
		h_total = h_ap + h_fp + h_sw + h_bp,
		v_al = 480,
		v_fp = 10,
		v_sw = 2,
		v_bp = 33,
		v_total = v_al + v_fp + v_sw + v_bp
		;
		
		
	/** TAKE TWO WITH PIPELINING **/
	
	// pixel counters
	logic X_CLEAR, Y_CLEAR;	// clear x, y counters
	logic [9:0] X, Y;			// x, y pixel positions
	logic [bitmap_widthbits-1:0] X_offset;
	logic [bitmap_heightbits-1:0] Y_offset;
	logic [text_buffer_widthbits-1:0] X_text;
	logic [text_buffer_heightbits-1:0] Y_text;
	
	// sync delays
	logic [vga_offset_bits-1:0] VGA_HS_SHIFT, VGA_VS_SHIFT, VGA_BLANK_SHIFT;
	assign VGA_HS = VGA_HS_SHIFT[vga_offset_bits-1];
	assign VGA_VS = VGA_VS_SHIFT[vga_offset_bits-1];
	assign VGA_BLANK_N = VGA_BLANK_SHIFT[vga_offset_bits-1];
	
	always_ff @(posedge VGA_CLK or negedge reset) begin: inc_x
		if (~reset) begin
			X <= 10'b0;
			X_text <= 0;
			X_offset <= 0;
		end else if (X_CLEAR) begin
			X <= 10'b0;
			X_text <= 0;
			X_offset <= 0;
		end else begin
			X <= X + 1'b1;
			if (X_offset == bitmap_width-1) begin
				X_offset <= 0;
				X_text <= X_text + 1'b1;
			end else begin
				X_offset <= X_offset + 1'b1;
			end
		end
	end
	assign X_CLEAR = (X == (h_total - 1));
	
	always_ff @(posedge VGA_CLK or negedge reset) begin: inc_y
		if (~reset) begin
			Y <= 10'b0;
			Y_text <= 0;
			Y_offset <= 0;
		end else if (X_CLEAR && Y_CLEAR) begin 
			Y <= 10'b0;
			Y_text <= 0;
			Y_offset <= 0;
		end else if (X_CLEAR) begin
			Y <= Y + 1'b1;
			if (Y_offset == bitmap_height-1) begin
				Y_offset <= 0;
				Y_text <= Y_text + 1'b1;
			end else begin
				Y_offset <= Y_offset + 1'b1;
			end
		end
	end
	assign Y_CLEAR = (Y == (v_total - 1));
	
	
	logic [13:0] text_buffer[0:text_buffer_width*text_buffer_height-1];
	
	always_ff @(posedge VGA_CLK) begin: sync_generator
		// two clock delay, for loading the pixels
		VGA_HS_SHIFT <= {VGA_HS_SHIFT[vga_offset_bits-2:0], ~((X >= h_ap + h_fp) & (X < h_ap + h_fp + h_sw))};
		VGA_VS_SHIFT <= {VGA_VS_SHIFT[vga_offset_bits-2:0], ~((Y >= v_al + v_fp) & (Y < v_al + v_fp + v_sw))};
		VGA_BLANK_SHIFT <= {VGA_BLANK_SHIFT[vga_offset_bits-2:0], ((X < h_ap) & (Y < v_al))};
	end
	
	// PIPELINE STAGE ZERO - Finding Addresses
	logic [text_buffer_addressbits-1:0] text_buffer_address;
	logic [bitmap_addressbits-1:0] bitmap_address_stage0;
	logic [text_buffer_widthbits-1:0] X_text_stage0;
	logic [text_buffer_heightbits-1:0] Y_text_stage0;
	always_ff @(posedge VGA_CLK) begin: calculate_text_buffer_address
		text_buffer_address <= Y_text * text_buffer_width + X_text;
		bitmap_address_stage0 <= Y_offset * bitmap_width + X_offset;
		X_text_stage0 <= X_text;
		Y_text_stage0 <= Y_text;
	end
	
	// PIPELINE STAGE ONE	
	logic [13:0] text_buffer_output;
	logic [bitmap_addressbits-1:0] bitmap_address_stage1;
	always_ff @(posedge VGA_CLK) begin: fetch_text_buffer_output
		text_buffer_output <= ((Y_text_stage0 < text_buffer_height) & (X_text_stage0 < text_buffer_width)) ? text_buffer[text_buffer_address] : 14'b0;
		bitmap_address_stage1 <= bitmap_address_stage0;
	end
	
	// PIPELINE STAGE TWO
	logic [0:bitmap_width*bitmap_height-1] bitmap;
	logic [2:0] background, foreground;
	logic [bitmap_addressbits-1:0] bitmap_address_stage2;
	always_ff @(posedge VGA_CLK) begin: fetch_bitmap
		unique case(text_buffer_output[7:0]) 
			8'h00 : bitmap = 48'h000000000000;
			8'h01 : bitmap = 48'h73EABE8B6700;
			8'h02 : bitmap = 48'h73E8B6DB6500;
			8'h03 : bitmap = 48'h014FBE708000;
			8'h04 : bitmap = 48'h00873E708000;
			8'h05 : bitmap = 48'h21C73ED88700;
			8'h06 : bitmap = 48'h20873EF88700;
			8'h07 : bitmap = 48'h00001CF9C000;
			8'h08 : bitmap = 48'hFFFFF3873FFF;
			8'h09 : bitmap = 48'h00C4A1852300;
			8'h0A : bitmap = 48'hFF3B5E7ADCFF;
			8'h0B : bitmap = 48'h386298924600;
			8'h0C : bitmap = 48'h72289C21C200;
			8'h0D : bitmap = 48'h30A208218600;
			8'h0E : bitmap = 48'h7924925B6C00;
			8'h0F : bitmap = 48'h02A73672A000;
			8'h10 : bitmap = 48'h01061C610000;
			8'h11 : bitmap = 48'h00431C304000;
			8'h12 : bitmap = 48'h21CF88F9C200;
			8'h13 : bitmap = 48'h514514014500;
			8'h14 : bitmap = 48'h7BAE9A28A280;
			8'h15 : bitmap = 48'h7A0722702F00;
			8'h16 : bitmap = 48'h00000003EF80;
			8'h17 : bitmap = 48'h21CA88200F80;
			8'h18 : bitmap = 48'h21CA88208200;
			8'h19 : bitmap = 48'h208208A9C200;
			8'h1A : bitmap = 48'h00813E108000;
			8'h1B : bitmap = 48'h00843E408000;
			8'h1C : bitmap = 48'h000084210F80;
			8'h1D : bitmap = 48'h00053E500000;
			8'h1E : bitmap = 48'h21CF9C71C000;
			8'h1F : bitmap = 48'h01C71CF9C200;
			8'h20 : bitmap = 48'h000000000000;
			8'h21 : bitmap = 48'h208208008200;
			8'h22 : bitmap = 48'h514500000000;
			8'h23 : bitmap = 48'h514F94F94500;
			8'h24 : bitmap = 48'h21EA1C2BC200;
			8'h25 : bitmap = 48'hC32108426180;
			8'h26 : bitmap = 48'h428A10AA4680;
			8'h27 : bitmap = 48'h208200000000;
			8'h28 : bitmap = 48'h108410408100;
			8'h29 : bitmap = 48'h408104108400;
			8'h2A : bitmap = 48'h008A9CA88000;
			8'h2B : bitmap = 48'h00823E208000;
			8'h2C : bitmap = 48'h000000608400;
			8'h2D : bitmap = 48'h00003E000000;
			8'h2E : bitmap = 48'h000000018600;
			8'h2F : bitmap = 48'h002108420000;
			8'h30 : bitmap = 48'h7229AACA2700;
			8'h31 : bitmap = 48'h218208208700;
			8'h32 : bitmap = 48'h722084210F80;
			8'h33 : bitmap = 48'hF842040A2700;
			8'h34 : bitmap = 48'h10C524F84100;
			8'h35 : bitmap = 48'hFA0F020A2700;
			8'h36 : bitmap = 48'h31083C8A2700;
			8'h37 : bitmap = 48'hF82108410400;
			8'h38 : bitmap = 48'h72289C8A2700;
			8'h39 : bitmap = 48'h72289E084600;
			8'h3A : bitmap = 48'h018600618000;
			8'h3B : bitmap = 48'h018600608400;
			8'h3C : bitmap = 48'h108420408100;
			8'h3D : bitmap = 48'h000F80F80000;
			8'h3E : bitmap = 48'h408102108400;
			8'h3F : bitmap = 48'h722084200200;
			8'h40 : bitmap = 48'h722AAEA20700;
			8'h41 : bitmap = 48'h7228BE8A2880;
			8'h42 : bitmap = 48'hF228BC8A2F00;
			8'h43 : bitmap = 48'h722820822700;
			8'h44 : bitmap = 48'hF228A28A2F00;
			8'h45 : bitmap = 48'hFA083C820F80;
			8'h46 : bitmap = 48'hFA083C820800;
			8'h47 : bitmap = 48'h72282E8A2700;
			8'h48 : bitmap = 48'h8A28BE8A2880;
			8'h49 : bitmap = 48'h708208208700;
			8'h4A : bitmap = 48'h384104124600;
			8'h4B : bitmap = 48'h8A4A30A24880;
			8'h4C : bitmap = 48'h820820820F80;
			8'h4D : bitmap = 48'h8B6AAA8A2880;
			8'h4E : bitmap = 48'h8A2CAA9A2880;
			8'h4F : bitmap = 48'h7228A28A2700;
			8'h50 : bitmap = 48'hF228BC820800;
			8'h51 : bitmap = 48'h7228A2AA4680;
			8'h52 : bitmap = 48'hF228BCA24880;
			8'h53 : bitmap = 48'h7A081C082F00;
			8'h54 : bitmap = 48'hF88208208200;
			8'h55 : bitmap = 48'h8A28A28A2700;
			8'h56 : bitmap = 48'h8A28A2514200;
			8'h57 : bitmap = 48'h8A28AAAAA500;
			8'h58 : bitmap = 48'h8A2508522880;
			8'h59 : bitmap = 48'h8A2894208200;
			8'h5A : bitmap = 48'hF82108420F80;
			8'h5B : bitmap = 48'h308208208300;
			8'h5C : bitmap = 48'h020408102000;
			8'h5D : bitmap = 48'h608208208600;
			8'h5E : bitmap = 48'h214880000000;
			8'h5F : bitmap = 48'h000000000F80;
			8'h60 : bitmap = 48'h410200000000;
			8'h61 : bitmap = 48'h0007027A2780;
			8'h62 : bitmap = 48'h820B328A2F00;
			8'h63 : bitmap = 48'h000720822700;
			8'h64 : bitmap = 48'h0826A68A2780;
			8'h65 : bitmap = 48'h000722FA0700;
			8'h66 : bitmap = 48'h312438410400;
			8'h67 : bitmap = 48'h01E8A2782700;
			8'h68 : bitmap = 48'h820B328A2880;
			8'h69 : bitmap = 48'h008018208700;
			8'h6A : bitmap = 48'h100304124600;
			8'h6B : bitmap = 48'h410494614480;
			8'h6C : bitmap = 48'h608208208700;
			8'h6D : bitmap = 48'h000D2AAAAA80;
			8'h6E : bitmap = 48'h000B328A2880;
			8'h6F : bitmap = 48'h0007228A2700;
			8'h70 : bitmap = 48'h000F22F20800;
			8'h71 : bitmap = 48'h0006A6782080;
			8'h72 : bitmap = 48'h000B32820800;
			8'h73 : bitmap = 48'h000720702F00;
			8'h74 : bitmap = 48'h410E10412300;
			8'h75 : bitmap = 48'h0008A28A6680;
			8'h76 : bitmap = 48'h0008A2894200;
			8'h77 : bitmap = 48'h0008A2AAA500;
			8'h78 : bitmap = 48'h000894214880;
			8'h79 : bitmap = 48'h0008A2782700;
			8'h7A : bitmap = 48'h000F84210F80;
			8'h7B : bitmap = 48'h188210208180;
			8'h7C : bitmap = 48'h208208208200;
			8'h7D : bitmap = 48'hC08204208C00;
			8'h7E : bitmap = 48'h010A84000000;
			8'h7F : bitmap = 48'h000008522F80;
			8'h80 : bitmap = 48'h722822708E00;
			8'h81 : bitmap = 48'h5008A28A6680;
			8'h82 : bitmap = 48'h108722FA0700;
			8'h83 : bitmap = 48'h2147027A2780;
			8'h84 : bitmap = 48'h5007027A2780;
			8'h85 : bitmap = 48'h4087027A2780;
			8'h86 : bitmap = 48'h30C7027A2780;
			8'h87 : bitmap = 48'h00072089CE00;
			8'h88 : bitmap = 48'h214722FA0700;
			8'h89 : bitmap = 48'h500722FA0700;
			8'h8A : bitmap = 48'h408722FA0700;
			8'h8B : bitmap = 48'h500018208700;
			8'h8C : bitmap = 48'h214018208700;
			8'h8D : bitmap = 48'h408018208700;
			8'h8E : bitmap = 48'h5007228BE880;
			8'h8F : bitmap = 48'h2147228BE880;
			8'h90 : bitmap = 48'h108FA0F20F80;
			8'h91 : bitmap = 48'h00070A7A8780;
			8'h92 : bitmap = 48'h7A8A3EA28B80;
			8'h93 : bitmap = 48'h21401C8A2700;
			8'h94 : bitmap = 48'h50001C8A2700;
			8'h95 : bitmap = 48'h40801C8A2700;
			8'h96 : bitmap = 48'h2140228A6680;
			8'h97 : bitmap = 48'h4088A28A6680;
			8'h98 : bitmap = 48'h5008A2782700;
			8'h99 : bitmap = 48'h5007228A2700;
			8'h9A : bitmap = 48'h5008A28A2700;
			8'h9B : bitmap = 48'h008728A9C200;
			8'h9C : bitmap = 48'h31243C420F80;
			8'h9D : bitmap = 48'h8A253E23E200;
			8'h9E : bitmap = 48'hE249389A4880;
			8'h9F : bitmap = 48'h10A21C228400;
			8'hA0 : bitmap = 48'h1087027A2780;
			8'hA1 : bitmap = 48'h108018208700;
			8'hA2 : bitmap = 48'h1087228A2700;
			8'hA3 : bitmap = 48'h1088A28A6680;
			8'hA4 : bitmap = 48'h29402CCA2880;
			8'hA5 : bitmap = 48'h2948B2AA6880;
			8'hA6 : bitmap = 48'h724700F80000;
			8'hA7 : bitmap = 48'h722700F80000;
			8'hA8 : bitmap = 48'h200210822700;
			8'hA9 : bitmap = 48'h00003E820000;
			8'hAA : bitmap = 48'h00003E082000;
			8'hAB : bitmap = 48'h8A4A14884180;
			8'hAC : bitmap = 48'h8A4A14B0E100;
			8'hAD : bitmap = 48'h208008208200;
			8'hAE : bitmap = 48'h000294A14280;
			8'hAF : bitmap = 48'h000A14294A00;
			8'hB0 : bitmap = 48'h264489912264;
			8'hB1 : bitmap = 48'h56A56A56A56A;
			8'hB2 : bitmap = 48'h6F6B5BDAD6F6;
			8'hB3 : bitmap = 48'h208208208208;
			8'hB4 : bitmap = 48'h208238208208;
			8'hB5 : bitmap = 48'h208E08E08208;
			8'hB6 : bitmap = 48'h514534514514;
			8'hB7 : bitmap = 48'h00003C514514;
			8'hB8 : bitmap = 48'h000E08E08208;
			8'hB9 : bitmap = 48'h514D04D14514;
			8'hBA : bitmap = 48'h514514514514;
			8'hBB : bitmap = 48'h000F04D14514;
			8'hBC : bitmap = 48'h514D04F00000;
			8'hBD : bitmap = 48'h51453C000000;
			8'hBE : bitmap = 48'h208E08E00000;
			8'hBF : bitmap = 48'h000038208208;
			8'hC0 : bitmap = 48'h20820F000000;
			8'hC1 : bitmap = 48'h20823F000000;
			8'hC2 : bitmap = 48'h00003F208208;
			8'hC3 : bitmap = 48'h20820F208208;
			8'hC4 : bitmap = 48'h00003F000000;
			8'hC5 : bitmap = 48'h20823F208208;
			8'hC6 : bitmap = 48'h2083C83C8208;
			8'hC7 : bitmap = 48'h514517514514;
			8'hC8 : bitmap = 48'h5145D07C0000;
			8'hC9 : bitmap = 48'h0007D05D4514;
			8'hCA : bitmap = 48'h514DC0FC0000;
			8'hCB : bitmap = 48'h000FC0DD4514;
			8'hCC : bitmap = 48'h5145D05D4514;
			8'hCD : bitmap = 48'h000FC0FC0000;
			8'hCE : bitmap = 48'h514DC0DD4514;
			8'hCF : bitmap = 48'h208FC0FC0000;
			8'hD0 : bitmap = 48'h51453F000000;
			8'hD1 : bitmap = 48'h000FC0FC8208;
			8'hD2 : bitmap = 48'h00003F514514;
			8'hD3 : bitmap = 48'h51451F000000;
			8'hD4 : bitmap = 48'h2083C83C0000;
			8'hD5 : bitmap = 48'h0003C83C8208;
			8'hD6 : bitmap = 48'h00001F514514;
			8'hD7 : bitmap = 48'h514537514514;
			8'hD8 : bitmap = 48'h208FC0FC8208;
			8'hD9 : bitmap = 48'h208238000000;
			8'hDA : bitmap = 48'h00000F208208;
			8'hDB : bitmap = 48'hFFFFFFFFFFFF;
			8'hDC : bitmap = 48'h000000FFFFFF;
			8'hDD : bitmap = 48'hE38E38E38E38;
			8'hDE : bitmap = 48'h1C71C71C71C7;
			8'hDF : bitmap = 48'hFFFFFF000000;
			8'hE0 : bitmap = 48'h0006A4924680;
			8'hE1 : bitmap = 48'h31249C492B00;
			8'hE2 : bitmap = 48'hFA2820820800;
			8'hE3 : bitmap = 48'h000F94514980;
			8'hE4 : bitmap = 48'hF90204210F80;
			8'hE5 : bitmap = 48'h0007A4924600;
			8'hE6 : bitmap = 48'h00092493A800;
			8'hE7 : bitmap = 48'h0007A820A100;
			8'hE8 : bitmap = 48'h21CAAAA9C200;
			8'hE9 : bitmap = 48'h3128BE8A4600;
			8'hEA : bitmap = 48'h7228A2514D80;
			8'hEB : bitmap = 48'h3102047A2700;
			8'hEC : bitmap = 48'h00052A500000;
			8'hED : bitmap = 48'h20872A708200;
			8'hEE : bitmap = 48'h000720F20700;
			8'hEF : bitmap = 48'h01C8A28A2880;
			8'hF0 : bitmap = 48'h03E03E03E000;
			8'hF1 : bitmap = 48'h208F8823E000;
			8'hF2 : bitmap = 48'h818198800F80;
			8'hF3 : bitmap = 48'h08CC0C080F80;
			8'hF4 : bitmap = 48'h004288208208;
			8'hF5 : bitmap = 48'h208208228400;
			8'hF6 : bitmap = 48'h00803E008000;
			8'hF7 : bitmap = 48'h010A8442A100;
			8'hF8 : bitmap = 48'h624918000000;
			8'hF9 : bitmap = 48'h000008708000;
			8'hFA : bitmap = 48'h000000200000;
			8'hFB : bitmap = 48'h388208A18200;
			8'hFC : bitmap = 48'h614514000000;
			8'hFD : bitmap = 48'h604210700000;
			8'hFE : bitmap = 48'h000E38E00000;
			8'hFF : bitmap = 48'h000000000000;
		endcase
		
		bitmap_address_stage2 <= bitmap_address_stage1;
		background <= text_buffer_output[10:8];
		foreground <= text_buffer_output[13:11];
	end
	
	// PIPELINE STAGE THREE
	logic [3:0] colour;
	always_ff @(posedge VGA_CLK) begin: set_colour
		colour <= bitmap[bitmap_address_stage2] ? foreground : background;
	end
	
	
	// SYNC signals
	assign VGA_SYNC_N = 1'b1;
	assign VGA_R = colour[2] ? 8'hFF : 8'h00;
	assign VGA_G = colour[1] ? 8'hFF : 8'h00;
	assign VGA_B = colour[0] ? 8'hFF : 8'h00;

endmodule