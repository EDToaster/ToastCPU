


module hex_driver(
	io_interface io,
	output [6:0] HEX0, HEX1, HEX2, HEX3
	);
	
	assign io.rdata = word;
	wire wenable = io.wenable & io.waddr == 16'b0;
	
	reg [15:0] word;
	
	always_ff @(posedge io.clock) begin
		if (wenable) word <= io.wdata;
	end
	
	display_word d_io(
		word,
		HEX0, HEX1, HEX2, HEX3
	);
	

endmodule