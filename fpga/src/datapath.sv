`include "types/control_signals_imports.svh"

// control signals for the datapath
// are borrowed from the MIPS standard
module datapath (
    input logic clock,
    input logic reset,					// reset low
    
    input logic reg_write,
    input logic mem_to_reg, 			// transfer memory to reg? (for load, etc)
    input logic mem_read_is_pc,			// set the read address 
    input logic mem_read_is_sp,			// set the read address 
    input alu_output_override_t::t alu_output_override,
    input logic alu_override_imm4,		// override input of alu_b to be imm4 value?
    input logic alu_set_flags,			// on clock, set status flags?
    input logic set_pc,					// on clock
    input pc_data_source_t::t pc_data_source,

    input logic sr_from_mem,
    input logic mem_write,
    
    input mem_write_addr_source_t::t mem_write_addr_source,
    input mem_write_data_source_t::t mem_write_data_source,	// set the memory write data to this source
        
    input logic set_sp,					// should we set the sp on this cycle?
    input logic increase_sp,			// should the set_sp be an increase?
    input logic reset_irq,
    
    input logic [3:0] register_addrpoke, 
    
    io_interface key_io,
    io_interface vga_io,
    
    output logic [15:0] current_instruction,
    //output logic do_halt,
    output logic Z_out,
    output logic N_out,
    output logic [15:0] PC_poke,
    output logic [15:0] mem_poke,
    output logic [15:0] register_datapoke,
    output logic irq,
    
    output logic [6:0] HEX0,
    output logic [6:0] HEX1,
    output logic [6:0] HEX2,
    output logic [6:0] HEX3
    
);
    // sp/sr/pc
    logic [15:0] SP, SR, PC;
    
    wire [15:0] next_PC;
    always_comb begin: next_PC_select
        unique case(pc_data_source)
            pc_data_source_t::irq,
            pc_data_source_t::register	: next_PC = reg_rdata1;
            pc_data_source_t::mem		: next_PC = mem_rdata;
            pc_data_source_t::next_pc	: next_PC = PC + 16'h1;
        endcase
    end

    wire [15:0] next_SP = increase_sp ? SP + 16'h1 : SP - 16'h1;
    
    assign PC_poke = PC;
    assign mem_poke = mem_rdata;
    
    // memory
    wire [15:0] mem_raddr, mem_rdata, mem_waddr, mem_wdata;
    wire mem_wenable = mem_write, mem_rvalid;
    
    assign mem_raddr = mem_read_is_pc ? PC : (mem_read_is_sp ? SP : reg_rdata2);	// always reading from pc sp or r2

    always_comb begin: mem_waddr_select
        unique case(mem_write_addr_source)
            mem_write_addr_source_t::register_data 	: mem_waddr = reg_rdata1;
            mem_write_addr_source_t::sp             : mem_waddr = SP;
        endcase
    end

    always_comb begin: mem_wdata_select
        unique case(mem_write_data_source) 
            mem_write_data_source_t::register_data	: mem_wdata = reg_rdata2; 
            mem_write_data_source_t::next_pc		: mem_wdata = next_PC;
            mem_write_data_source_t::this_pc		: mem_wdata = PC;
            mem_write_data_source_t::sr				: mem_wdata = SR;
        endcase
    end
    
//	assign do_halt = opcode == 4'b0111;
    assign Z_out = SR[1];
    assign N_out = SR[2];
    
    memory_control memory(	
        .clock,		
        .read_address(mem_raddr),
        .write_address(mem_waddr),
        .write_data(mem_wdata),
        .write_enable(mem_wenable),
        .read_data(mem_rdata),
        .read_valid(mem_rvalid),
        
        .key_io,
        .vga_io
    );
    
    assign key_io.reset_irq = reset_irq;
    assign vga_io.reset_irq = reset_irq;
    assign irq = key_io.irq | vga_io.irq;
    
    wire [3:0]  opcode = current_instruction[15:12];
    wire [3:0]  r1 = current_instruction[11:8], 
                     r2 = current_instruction[7:4],
                     jump_reg = r1;
    wire [3:0]  alu_op = current_instruction[3:0];
    // sign extend immediate value to 16 bits
    wire [3:0] 	imm4 = current_instruction[7:4];
    wire [7:0]  imm8 = current_instruction[7:0];
    wire [15:0] imm4_16 = { 12'b0 , imm4[3:0] };
    
    
    // registers
    wire [7:0] reg_raddr1 = (pc_data_source == pc_data_source_t::irq) ? 4'hC : r1, reg_raddr2 = r2;
    wire [15:0] reg_rdata1, reg_rdata2;
    wire [7:0] reg_waddr = r1;		// always will be writing to r1
    wire [15:0] reg_wdata = mem_to_reg ? mem_rdata : alu_out;
    
//	display_word(
//		mem_rdata,
//		HEX0, HEX1, HEX2, HEX3
//	);
    
    registers register_file(
        .clock,
        
        // todo: set back select based on SR
        
        // reading
        .read_addr1(reg_raddr1), 	
        .read_addr2(reg_raddr2),
        .read_data1(reg_rdata1), 				
        .read_data2(reg_rdata2),
        .read_addrpoke(register_addrpoke),
        .read_datapoke(register_datapoke),
        
        // writing
        .write_addr(reg_waddr), 
        .write_data(reg_wdata), 
        .write_en(reg_write),
        
        // 13, 14, 15
        // SP, SR, PC are accessible from registers but 
        // are read-only
        .SP, .SR, .PC
    );
    
    
    // alu
    wire [15:0] a_in = reg_rdata1, b_in = alu_override_imm4 ? imm4_16 : reg_rdata2;
    wire [15:0] alu_out;
    
    wire V, C, N, Z, X, set_VC;

    // if the instruction is one of 4'b1000, we set appropriate flags
    alu datapath_alu(
        .a_in,
        .b_in,
        
        .output_override(imm8),
        .alu_output_override,
        
        .operation(alu_op),
        // todo: add carry support, including carry_enable
        .carry_in(1'b0),
        
        // outputs
        .agg(alu_out),
        
        .V,		// overflow
        .C,		// carry
        .N,		// negative
        .Z,		// zero
        .X,		// ffff
        .set_VC	// should set V, C flag?
    );

    
    // reset routine
    always_ff @(posedge clock or negedge reset)
    begin
        if (~reset) begin
            SP <= 16'h8000;
            SR <= 16'h0000;
            PC <= 16'h0000;
        end else begin
            if (mem_read_is_pc) begin
                current_instruction <= mem_rdata;
            end 
            
            if (alu_set_flags) begin
                if (set_VC) SR[4:3] <= {V, C};
                SR[2:0] <= {N, Z, X};
            end

            if (sr_from_mem) begin
                SR <= mem_rdata;
            end
            
            if (set_pc) begin
                PC <= next_PC;
            end
            
            if (set_sp) begin
                SP <= next_SP;
            end

        end
    end
    

endmodule