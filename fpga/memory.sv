module memory_control (
	input clock,
	input [15:0] read_address,
	input [15:0] write_address,
	input [15:0] write_data,
	input write_enable,
	output [15:0] read_data,
	output read_valid
);

	// for now, set read_valid to 1 
	assign read_valid = 1'b1;

	logic [15:0] rom_data, ram_data, io_data;
	
	// rom
	rom program_memory(
		.address(read_address[14:0]),
		.clock,
		.q(rom_data)
	);
	
	// ram
	wire ram_write_en = write_enable & (write_address[15:14] == 2'b10);
	ram program_data(
		.byteena_a(2'b11),
		.clock,
		.data(write_data),
		.rdaddress(read_address[13:0]),
		.wraddress(write_address[13:0]),
		.wren(ram_write_en),
		.q(ram_data)
	);
	
	// io!
	
	
	always_comb begin: output_select
		unique case(read_address[15:14])
			2'b00, 2'b01:
				read_data = rom_data;
			2'b10:
				read_data = ram_data;
			2'b11:
				read_data = 16'h0000;
		endcase
	end
endmodule
