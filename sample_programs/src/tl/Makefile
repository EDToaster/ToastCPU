ifeq ($(OS),Windows_NT)
  # Windows
  RM := del
else
  # Not Windows
  RM := rm
endif

.PRECIOUS: all
.PHONY: all

os: os.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/os.mif

snake: snake.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/snake.mif

memset: memset.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/memset.mif

isr: isr.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/isr.mif

structs: structs.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/structs.mif

sigs: sigs.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/sigs.mif

test: test.mif
	cd ../../../emu && cargo run --package emu --bin emu -- ../sample_programs/src/tl/test.mif

%.mif: %.tasm
	python3 ../../../assembler/assembler.py -i $< -o $@

%.tasm: %.tl
	cd ../../../compiler && cargo run --package compiler --bin compiler -- -i ../sample_programs/src/tl/$< -o ../sample_programs/src/tl/$@ -I../sample_programs/lib/

clean:
	$(RM) *.mif *.tasm