.SECONDARY:
.PHONY:

MIFS = $(shell echo $(wildcard *.tl rosetta/*.tl) | sed -e "s/\\.tl/\\.mif/g")

# Extra emulator flags, e.g. `make life EMU_FLAGS=--window`
EMU_FLAGS ?=

%: %.mif
	cargo run --release --manifest-path ../../../emu/Cargo.toml -- $(EMU_FLAGS) $<

%.mif: %.tasm
	python3 ../../../assembler/assembler.py -i $< -o $@

%.tasm: %.tl ./tests/*.tl
	cargo run --manifest-path ../../../compiler/Cargo.toml -- -i $< -o $@ -I../../lib/ -I./ -v

clean:
	rm *.mif *.tasm rosetta/*.mif rosetta/*.tasm ||:

all: $(MIFS)