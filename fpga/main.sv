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
	wire	 slow_clock = counter[23];
	
	always_ff @(posedge CLOCK_50 or negedge reset)
	begin: clock_divide
		if (~reset) begin
			counter <= 24'd0;
		end else begin
			counter = counter - 1'b1;
		end
	
	end
	
	// increase bank number
	logic [7:0] bank_select = 4'h0;
	display_byte d_bank(
		{bank_select},
		HEX4, HEX5
	);

	always_ff @(posedge slow_clock) 
	begin: increment_bank
		bank_select = bank_select + 1'b1;
	end
	
	wire [15:0] read_data1, read_data2;
	
	// add registers
	registers register_file(
		.clock(CLOCK_50),
		.bank_select({6'b0, bank_select[1:0]}),
		
		// reading
		.read_addr1(SW[9:6]), 	.read_addr2(4'b0),
		.read_data1, 				.read_data2,
		
		// writing
		.write_addr(SW[9:6]), 
		.write_data({ 10'b0, SW[5:0] }), 
		.write_en(KEY[1]),
		
		// 13, 14, 15
		// SP, SR, PC are accessible from registers but 
		// are read-only
		.SP(16'hD), .SR(16'hE), .PC(16'hF)
	);
	
	display_word d_register(
		read_data1,
		HEX0, HEX1, HEX2, HEX3
	);
	
	
	
endmodule