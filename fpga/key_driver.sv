module key_driver(
	input logic reset,
	input logic CLOCK_50,
   input logic pc,
   input logic pd,
	io_interface io
);
    reg [15:0] denoise;
    wire pcf;
    assign pcf = (&denoise) ? 1'b1 : ((~|denoise) ? 1'b0 : pcf);

    always @(posedge CLOCK_50)
    begin
        denoise <= {denoise[14:0], pc};
    end

    // ** RAW SHIFTED DATA ** //
    reg [11:0] data;
    // ** CURRENT SCAN CODE BUFFER ** //
    reg [23:0] scan;

    reg prev_pcf;
	 
	 // 24-bit scan code
	 reg [23:0] out;
	 reg irq;
	 
	 // assign io
	 assign io.rdata = out[15:0];
	 assign io.irq = irq;

    always @(negedge CLOCK_50)
    begin
        // ** RESET LOGIC ** //
        if(~reset) 
        begin
            scan <= 24'h000000;
            data = 11'b11111111111;
				irq = 1'b0;
        end
		  else
		  begin
				if(io.reset_irq)
				begin
					 irq = 1'b0;
				end
		  end

        if(pcf == 1'b0 && prev_pcf == 1'b1) 
        begin
            // shift right
            data = {pd, data[11:1]};
            // if all shifted in
            if(data[0] == 1'b0)
            begin
                case(data[9:2])
                    8'hE0: scan[23:16] = data[9:2];
                    8'hF0: scan[15:8] = data[9:2];
                    default: begin
                        // full code
                        scan[7:0] = data[9:2];
                        out = scan;
                        
                        if(scan[15:8] == 8'hF0)
                        begin
                            // break code
                            // nothing yet, should set 
                        end
                        else
                        begin
                            // make code
									 // assert irq when we have a make code
									 irq = 1'b1;
                        end
                        
                        scan = 24'h000000;
                    end
                endcase

                data = 11'b11111111111;
            end
        end

        prev_pcf = pcf;
    end

endmodule