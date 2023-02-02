.SECONDARY:
.PHONY:

%: %.mif
	cargo run --release --manifest-path ../../../emu/Cargo.toml -- $<

%.mif: %.tasm
	python3 ../../../assembler/assembler.py -i $< -o $@

%.tasm: %.tl ./tests/*.tl
	cargo run --manifest-path ../../../compiler/Cargo.toml -- -i $< -o $@ -I../../lib/ -I./

clean:
	rm *.mif *.tasm