ifeq ($(OS),Windows_NT)
  # Windows
  RM := del
else
  # Not Windows
  RM := rm
endif

all: mandelbrot-small.mif fib.mif life.mif simple.mif


%: %.mif
	cargo run --release --manifest-path ../../../../emu/Cargo.toml -- $<

%.mif: %.tasm
	python3 ../../../../assembler/assembler.py -i $^ -o $@

%.tasm: %.b
	python3 bfcomp.py -i $^ -o $@

clean: 
	$(RM) *.mif
