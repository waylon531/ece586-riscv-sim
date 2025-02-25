%.elf: %.o
	riscv32-unknown-elf-ld -T linker.ld -o $@ $^

%.mem: %.elf
	./make_mem.sh $<

%.o: %.c
	riscv32-unknown-elf-gcc -fpic -march=rv32i -mabi=ilp32 -o $@ -c $^ 

%.o: %.s
	riscv32-unknown-elf-as -fpic -march=rv32i -mabi=ilp32 -o $@ -c $^ 
