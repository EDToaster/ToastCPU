

.PHONY: tl
tl: tl.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/tl.mif

.PHONY: out.mif
tl.mif: tl.tasm
	python3 ../../../assembler/assembler.py -i tl.tasm -o tl.mif

.PHONY: tl.tasm
tl.tasm: tl.tl
	cd ../../../compiler && cargo run --package compiler --bin compiler -- -i ../sample_programs/src/tl/tl.tl -o ../sample_programs/src/tl/tl.tasm


.PHONY: tl
sigs: sigs.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/sigs.mif

.PHONY: out.mif
sigs.mif: sigs.tasm
	python3 ../../../assembler/assembler.py -i sigs.tasm -o sigs.mif

.PHONY: sigs.tasm
sigs.tasm: sigs.tl
	cd ../../../compiler && cargo run --package compiler --bin compiler -- -i ../sample_programs/src/tl/sigs.tl -o ../sample_programs/src/tl/sigs.tasm

