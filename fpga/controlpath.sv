module control_path(

	input [15:0] instruction,
	
	input clock,
	input reset // reset low
);
	// halt is set on ALU fault or halt command
	enum { fetch, decode, execute, halt } curr_state, next_state;
	
	
	always_ff @(posedge clock or negedge reset) begin: reset_logic
		if (~reset) 
		begin
			// reset logic
			curr_state = fetch;
		end else begin
			// clocked logic
			curr_state = next_state;
		end
	end
	
	
	always_comb begin: next_state_logic
		next_state = curr_state;
		unique case(curr_state)
			// todo next state
			default:;
		endcase
	end

endmodule