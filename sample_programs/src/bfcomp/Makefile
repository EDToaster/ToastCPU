

out.mif: out.tasm
	python3 ../../../assembler/assembler.py -i out.tasm -o out.mif

out.tasm: mandelbrot-small.b
	python3 bfcomp.py -i mandelbrot-small.b -o out.tasm