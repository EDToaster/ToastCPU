

.PHONY: run
run: out.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/out.mif

.PHONY: out.mif
out.mif: ./**/*.tasm tl.tasm
	python3 ../../../assembler/assembler.py -i tl.tasm -o out.mif

.PHONY: tl.tasm
tl.tasm: tl.tl
	cd ../../../compiler && cargo run --package compiler --bin compiler -- -i ../sample_programs/src/tl/tl.tl -o ../sample_programs/src/tl/tl.tasm

