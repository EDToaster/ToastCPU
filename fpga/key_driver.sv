//module key_driver(
//	input logic reset,
//	input logic CLOCK_50,
//   input logic pc,
//   input logic pd,
//	io_interface io,
//	output logic [6:0] HEX4,
//	output logic [6:0] HEX5
//);
//    reg [15:0] denoise;
//    wire pcf;
//    assign pcf = (&denoise) ? 1'b1 : ((~|denoise) ? 1'b0 : pcf);
//
//    always @(posedge CLOCK_50)
//    begin
//        denoise <= {denoise[14:0], pc};
//    end
//
//    // ** RAW SHIFTED DATA ** //
//    reg [11:0] data;
//    // ** CURRENT SCAN CODE BUFFER ** //
//    reg [23:0] scan;
//
//    reg prev_pcf;
//	 
//	 // 24-bit scan code
//	 reg [23:0] out;
//	 reg irq;
//	 
//	 // assign io
//	 assign io.rdata = out[15:0];
//	 assign io.irq = irq;
//
//	 reg [7:0] debug_counter;
//	 
//	 	
//	display_byte disp_counter(
//		debug_counter,
//		HEX4, HEX5
//	);
//	 
//    always @(negedge CLOCK_50)
//    begin
//        // ** RESET LOGIC ** //
//        if(~reset) 
//        begin
//            scan <= 24'h000000;
//            data = 11'b11111111111;
//				irq = 1'b0;
//				
//				debug_counter = 8'b0;
//        end
//		  else
//		  begin
//				if(io.reset_irq)
//				begin
//					 irq = 1'b0;
//				end
//		  end
//
//        if(pcf == 1'b0 && prev_pcf == 1'b1) 
//        begin
//            // shift right
//             data = {pd, data[11:1]};
//            // if all shifted in
//            if(data[0] == 1'b0)
//            begin
//                case(data[9:2])
//                    8'hE0: scan[23:16] = data[9:2];
//                    8'hF0: scan[15:8] = data[9:2];
//                    default: begin
//                        // full code
//                        scan[7:0] = data[9:2];
//                        out = scan;
//                        
//                        if(scan[15:8] == 8'hF0)
//                        begin
//                            // break code
//                            // nothing yet, should set 
//                        end
//                        else
//                        begin
//                            // make code
//									 // assert irq when we have a make code
//									 irq = 1'b1;
//									 debug_counter = debug_counter + 1'b1;
//                        end
//                        
//                        scan = 25'h000000;
//                    end
//                endcase
//
//                data = 11'b11111111111;
//            end
//        end
//
//        prev_pcf = pcf;
//    end
//
//endmodule


// SOURCE:
// https://students.iitk.ac.in/eclub/assets/tutorials/keyboard.pdf
module key_driver (
	input logic clk, // Clock pin form keyboard
	input logic data, //Data pin form keyboard
	input logic reset,
	output logic [6:0] HEX4,
	output logic [6:0] HEX5,
	io_interface io
);

	reg [7:0] data_curr;
	reg [7:0] data_pre;
	reg [3:0] b;
	reg flag;
	
	reg irq;
	assign io.rdata = {8'b0, led};
	assign io.irq = irq;
	
	reg [7:0] led;
	reg [7:0] counter;
	
	display_byte disp_counter(
		counter,
		HEX4, HEX5
	);
	
	initial begin
		b <= 4'h1;
		flag <= 1'b0;
		data_curr <= 8'hf0;
		data_pre <= 8'hf0;
		led <= 8'hf0;
		counter <= 8'b0;
		irq <= 1'b0;
	end
	
	always @(negedge clk) //Activating at negative edge of clock from keyboard
	begin
		case(b)
			1:; //first bit
			2:data_curr[0]<=data;
			3:data_curr[1]<=data;
			4:data_curr[2]<=data;
			5:data_curr[3]<=data;
			6:data_curr[4]<=data;
			7:data_curr[5]<=data;
			8:data_curr[6]<=data;
			9:data_curr[7]<=data;
			10:flag<=1'b1; //Parity bit
			11:flag<=1'b0; //Ending bit
		endcase
		
		if(b<=10)
			b <= b + 1;
		else if(b == 11)
			b <= 1;
	end
	
	always@(posedge flag or posedge io.reset_irq) // Printing data obtained to led
	begin
		if (io.reset_irq)
		begin
			irq <= 1'b0;
		end
		else
		begin		
			if(data_curr==8'hf0)
			begin
				counter <= counter + 1'b1;
				led <= data_pre;
				irq <= 1'b1;
			end
			else
				data_pre <= data_curr;
			end
		end
 
endmodule