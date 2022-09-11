

%.mif: %.tasm
	python3 ../../../assembler/assembler.py -i $^ -o $@

%.tasm: %.b
	python3 bfcomp.py -i $^ -o $@