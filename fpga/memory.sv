module memory_control (
	input clock,
	input [15:0] read_address,
	input [15:0] write_address,
	input [15:0] write_data,
	input write_enable,
	output [15:0] read_data,
	output read_valid,
	
	io_interface hex_io, 
	io_interface vga_io,
	io_interface key_io,
		
	output logic [6:0] HEX0,
	output logic [6:0] HEX1,
	output logic [6:0] HEX2,
	output logic [6:0] HEX3
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
	

//	display_word(
//		ram_data,
//		HEX0, HEX1, HEX2, HEX3
//	);
	
	// io!
	
	// key = 16'hFFFF;
	// vga = 16'hC000;
	
	assign key_io.clock = clock;
	assign key_io.waddr = write_address - 16'b1111111111111111;
	assign key_io.wdata = write_data;
	assign key_io.wenable = write_enable & (write_address == 16'b1111111111111111);
	
	assign vga_io.clock = clock;
	assign vga_io.waddr = write_address - 16'b1100000000000000;
	assign vga_io.wdata = write_data;
	assign vga_io.wenable = write_enable & (write_address[15:14] == 2'b11);
	
	
	always_comb begin: output_select
		if (read_address[15] == 1'b0)
		begin
			read_data = rom_data;
		end
		else if (read_address[15:14] == 2'b10)
		begin
			read_data = ram_data;
		end
		else if (read_address[15:14] == 3'b110)
		begin
			read_data = vga_io.rdata;
		end
		else
		begin
			read_data = key_io.rdata;
		end
		
//		unique casez(read_address)
//			16'b0???????????????:
//				read_data = rom_data;
//			16'b10??????????????:
//				read_data = ram_data;
//			16'b110??????????????:
//				read_data = vga_io.rdata;
//			16'h111??????????????:
//				read_data = key_io.rdata;
//			default:
//				read_data = 16'b0;
//		endcase
	end
endmodule
