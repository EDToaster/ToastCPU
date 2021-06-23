

// control signals for the datapath
// are borrowed from the MIPS standard
module datapath (
	input logic clock,
	input logic reset,					// reset low
	
	input logic reg_write,
	input logic mem_to_reg, 			// transfer memory to reg? (for load, etc)
	input logic fetch_instruction,	// on clock
	input logic alu_override_imm,		// override output of alu to be imm16 value?
	input logic alu_override_b,		// override input b of alu to be 1?
	input logic alu_set_flags,			// on clock, set status flags?
	input logic set_pc,					// on clock
	input logic pc_from_register,		
	input logic mem_write,
	
	input logic [3:0] register_addrpoke, 
	
	io_interface hex_io,
	io_interface vga_io,
	
	output logic [15:0] current_instruction,
	//output logic do_halt,
	output logic Z_out,
	output logic [15:0] PC_poke,
	output logic [15:0] mem_poke,
	output logic [15:0] register_datapoke
);


	// sp/sr/pc
	logic [15:0] SP, SR, PC;
	wire [15:0] next_PC = pc_from_register ? reg_rdata1 : PC + 16'h1;
	
	assign PC_poke = PC;
	assign mem_poke = mem_rdata;
	
	// memory
	wire [15:0] mem_raddr, mem_rdata, mem_waddr, mem_wdata;
	wire mem_wenable = mem_write, mem_rvalid;
	
	assign mem_raddr = fetch_instruction ? PC : reg_rdata2;	// always reading from pc or r2
	assign mem_waddr = reg_rdata1;									// always writing to r1
	assign mem_wdata = reg_rdata2;									// always writing data from r2
	
//	assign do_halt = opcode == 4'b0111;
	assign Z_out = SR[1];
	
	memory_control memory(
		.clock,		
		.read_address(mem_raddr),
		.write_address(mem_waddr),
		.write_data(mem_wdata),
		.write_enable(mem_wenable),
		.read_data(mem_rdata),
		.read_valid(mem_rvalid),
		
		.hex_io,
		.vga_io
	);
	
	wire [3:0]  opcode = current_instruction[15:12];
	wire [3:0]  r1 = current_instruction[11:8], 
					 r2 = current_instruction[7:4],
					 jump_reg = r1;
	wire [3:0]  alu_op = current_instruction[3:0];
	// sign extend immediate value to 16 bits
	wire [7:0]  imm8 = current_instruction[7:0];
	wire [15:0] imm16 = { {8{imm8[7]}}, imm8[7:0] }; 	
	
	
	// registers
	wire [7:0] reg_raddr1 = r1, reg_raddr2 = r2;
	wire [15:0] reg_rdata1, reg_rdata2;
	wire [7:0] reg_waddr = r1;		// always will be writing to r1
	wire [15:0] reg_wdata = mem_to_reg ? mem_rdata : alu_out;
	
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
	wire [15:0] a_in = reg_rdata1, b_in = alu_override_b ? 16'h1 : reg_rdata2;
	wire [15:0] alu_out;
	
	wire V, C, N, Z, X, set_VC;

	// if the instruction is one of 4'b1000, we set appropriate flags
	alu datapath_alu(
		.a_in,
		.b_in,
		.override(imm16),
		.override_en(alu_override_imm),
		
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
//			input fetch_instruction,
//			input alu_set_flags,			// set status flags?
//			input set_pc,
	
			if (fetch_instruction) begin
				current_instruction <= mem_rdata;
			end 
			
			if (alu_set_flags) begin
				if (set_VC) SR[4:3] <= {V, C};
				SR[2:0] <= {N, Z, X};
			end
			
			if (set_pc) begin
				PC <= next_PC;
			end

		end
	end
	

endmodule