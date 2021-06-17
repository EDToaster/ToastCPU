module alu (
	input [15:0] a_in,
	input [15:0] b_in,
	
	input [3:0] select,
	input carry_in,
	
	// outputs
	output [15:0] agg,
	
	output V,		// overflow
	output C,		// carry
	output N,		// negative
	output Z,		// zero
	output X,		// ffff
	output set_VC	// should set V, C flag?
);

	localparam
		NOT = 3'h0,
		AND = 3'h1,
		OR	 = 3'h2,
		XOR = 3'h3,
		ADD = 3'h4,
		SUB = 3'h5;

	// if msb of select is a 1, then 
	wire [2:0] operation = select[2:0];
	wire override = select[3];
	wire [15:0] a = a_in;
	wire [15:0] b = override ? 16'b1 : b_in;
	wire [16:0] inter_agg;
	
	assign agg = inter_agg[15:0];
	assign set_VC = operation[2];
	
	
	always_comb
	begin: alu_V
		case(operation)
			ADD: V = (a[15] == b[15]) & (a[15] ^ agg[15]);
			SUB: V = (a[15] ^ b[15]) & (a[15] != agg[15]);
			default: V = 1'b0;
		endcase
	end
	
	always_comb
	begin: alu_C
		case(operation)
			ADD, SUB: C = inter_agg[16];
			default: C = 1'b0;
		endcase
	end
	
	assign N = agg[15];
	assign Z = agg == 16'h0000;
	assign X = agg == 16'hFFFF;

	always_comb
	begin: alu_comb
		case(operation)
			NOT: inter_agg = ~a;
			AND: inter_agg = a & b;
			OR : inter_agg = a | b;
			XOR: inter_agg = a ^ b;
			ADD: inter_agg = a + b;
			SUB: inter_agg = a - b;
			default: inter_agg = 17'h0;
		endcase
	end
	


endmodule