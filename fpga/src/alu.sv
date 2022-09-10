`include "types/control_signals_imports.svh"

module alu (
    input [15:0] a_in,
    input [15:0] b_in,
    
    input [7:0] output_override,
    input alu_output_override_t::t alu_output_override,
    
    input [3:0] operation,
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
        NOT = 4'h0,
        AND = 4'h1,
        OR  = 4'h2,
        XOR = 4'h3,
        ADD = 4'h4,
        SUB = 4'h5,
        MOV = 4'h6,
        CMP = 4'h7,
        SHR = 4'h8,
        SSHR= 4'h9,
        SHL = 4'hA;

    // if msb of select is a 1, then 
    wire [15:0] a = a_in;
    wire [15:0] b = b_in;
    wire [16:0] inter_agg;
    
    assign agg = inter_agg[15:0];
    assign set_VC = operation[2];
    
    
    always_comb
    begin: alu_V
        if (alu_output_override != alu_output_override_t::none) begin V = 1'b0; end
        else begin 
            case(operation)
                ADD: V = (a[15] == b[15]) & (a[15] ^ agg[15]);
                SUB, CMP: V = (a[15] ^ b[15]) & (a[15] != agg[15]);
                default: V = 1'b0;
            endcase
        end
    end
    
    always_comb
    begin: alu_C
        if (alu_output_override != alu_output_override_t::none) begin C = 1'b0; end
        else begin 
            case(operation)
                ADD, SUB, CMP: C = inter_agg[16];
                default: C = 1'b0;
            endcase
        end
    end
    
    assign N = agg[15];
    assign Z = agg == 16'h0000;
    assign X = agg == 16'hFFFF;

    always_comb
    begin: alu_comb
        if (alu_output_override == alu_output_override_t::imm8) begin 
            inter_agg = { { 9{output_override[7]} }, output_override }; 
        end
        else if (alu_output_override == alu_output_override_t::imm8_high) begin 
            inter_agg = { output_override[7], output_override[7:0], a_in[7:0] }; 
        end
        else begin 
            case(operation)
                NOT: inter_agg = ~a;
                AND: inter_agg = a & b;
                OR : inter_agg = a | b;
                XOR: inter_agg = a ^ b;
                ADD: inter_agg = a + b;
                SUB, CMP: inter_agg = a - b;
                MOV: inter_agg = b;
                SHR: inter_agg = a >> b;
                SSHR:inter_agg = a >>> b;
                SHL: inter_agg = a << b;
                default: inter_agg = 17'h0;
            endcase
        end
    end
endmodule