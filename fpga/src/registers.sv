module read_register(
    input [3:0] addr,
    input [15:0] banked[8],	// banked in the future?
    input [15:0] scratch[4],
    input [15:0] ISR, SP, SR, PC,
    
    output [15:0] data
);

    always_comb begin: read_select
        unique case(addr)
            4'h0, 
            4'h1, 
            4'h2, 
            4'h3, 
            4'h4, 
            4'h5, 
            4'h6, 
            4'h7: data = banked[addr[2:0]];
            4'h8,
            4'h9, 
            4'hA, 
            4'hB: data = scratch[addr[1:0]];
            4'hC: data = ISR;
            4'hD: data = SP;
            4'hE: data = SR;
            4'hF: data = PC;
        endcase
    end
endmodule


module registers (
    input clock,
    
    // reading
    input [3:0] read_addr1, read_addr2, read_addrpoke,
    output [15:0] read_data1, read_data2, read_datapoke,
    
    // writing
    input [3:0] write_addr,
    input [15:0] write_data,
    input write_en,
    
    // 14, 15
    // SR, PC are accessible from registers but 
    // are read-only
    input [15:0] SR, PC,

    input reset
);
    
    reg [15:0] banked[8];
    
    reg [15:0] ISR;
    reg [15:0] SP;
    
    reg [15:0] scratch[4];
    
    read_register read1(
        read_addr1,
        banked,
        scratch,
        ISR, SP, SR, PC,
        read_data1
    );
    
    read_register read2(
        read_addr2,
        banked,
        scratch,
        ISR, SP, SR, PC,
        read_data2
    );
    
    read_register readpoke(
        read_addrpoke,
        banked,
        scratch,
        ISR, SP, SR, PC,
        read_datapoke
    );
    
    always_ff @(posedge clock or negedge reset) begin: write_cycle
        if (~reset) begin
            SP <= 16'hBFFF;
        end else begin
            if (write_en) begin
                case(write_addr)
                    4'h0, 
                    4'h1, 
                    4'h2, 
                    4'h3, 
                    4'h4, 
                    4'h5, 
                    4'h6, 
                    4'h7: banked[write_addr[2:0]][15:0] <= write_data;
                    4'h8,
                    4'h9, 
                    4'hA, 
                    4'hB: scratch[write_addr[1:0]][15:0] <= write_data;
                    4'hC: ISR <= write_data;
                    4'hD: SP <= write_data;
                    default:; // no op for writing to SR, PC
                endcase
            end
        end
    end

endmodule