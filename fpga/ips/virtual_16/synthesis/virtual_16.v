// virtual_16.v

// Generated using ACDS version 21.1 850

`timescale 1 ps / 1 ps
module virtual_16 (
		input  wire [15:0] probe,  //  probes.probe
		output wire [15:0] source  // sources.source
	);

	altsource_probe_top #(
		.sld_auto_instance_index ("YES"),
		.sld_instance_index      (0),
		.instance_id             ("NONE"),
		.probe_width             (16),
		.source_width            (16),
		.source_initial_value    ("0"),
		.enable_metastability    ("NO")
	) in_system_sources_probes_0 (
		.source     (source), // sources.source
		.probe      (probe),  //  probes.probe
		.source_ena (1'b1)    // (terminated)
	);

endmodule
