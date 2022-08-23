module switch_driver(
	input logic reset,
	input logic CLOCK_50,
   	input key,
	io_interface io
); 
	 reg [3:0] prev_key;
	 reg irq;
	 
	 
	 assign io.rdata = {15'b0, key};
	 assign io.irq = irq;

    always @(posedge key or posedge io.reset_irq)
    begin
		if (io.reset_irq)
		begin
			irq = 1'b0;
		end
		else 
		begin
			irq = 1'b1;
		end
	 end

endmodule