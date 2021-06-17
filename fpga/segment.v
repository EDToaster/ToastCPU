module segment(
    input [3:0] select,
    output reg [6:0] hex
);

    always @(*)
    begin: lut
        case(select)
            4'h0: hex <= ~7'h3F;
            4'h1: hex <= ~7'h06;
            4'h2: hex <= ~7'h5B;
            4'h3: hex <= ~7'h4F;
            4'h4: hex <= ~7'h66;
            4'h5: hex <= ~7'h6D;
            4'h6: hex <= ~7'h7D;
            4'h7: hex <= ~7'h07;
            4'h8: hex <= ~7'h7F;
            4'h9: hex <= ~7'h6F;
            4'hA: hex <= ~7'h77;
            4'hB: hex <= ~7'h7C;
            4'hC: hex <= ~7'h39;
            4'hD: hex <= ~7'h5E;
            4'hE: hex <= ~7'h79;
            4'hF: hex <= ~7'h71;
        endcase
    end

endmodule

module display_byte(
	input [7:0] byte,
	output [6:0] h0,
	output [6:0] h1
);

	segment d0(
		byte[3:0],
		h0
	);
	
	segment d1(
		byte[7:4],
		h1
	);
	
endmodule

module display_word(
	input [15:0] word,
	output [6:0] h0,
	output [6:0] h1,
	output [6:0] h2,
	output [6:0] h3
);
	display_byte d0(
		word[7:0],
		h0,
		h1
	);
	
	display_byte d1(
		word[15:8],
		h2,
		h3
	);
	
endmodule