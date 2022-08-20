
package mem_write_addr_source_t;
	typedef enum { 
		register_data,
		sp
	} t;
endpackage

package mem_write_data_source_t;
	typedef enum { 
		register_data,
		next_pc,
		this_pc, 
		sr
	} t;
endpackage