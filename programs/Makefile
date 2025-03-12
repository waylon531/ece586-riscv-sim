RVGCC:=riscv64-unknown-elf-gcc
RVLD:=$(RVGCC)
#RVLD:=riscv32-unknown-elf-ld
RVAS:=riscv64-unknown-elf-as
RVSTRIP:=riscv64-unknown-elf-strip
CFLAGS:=-fpic -march=rv32i -mabi=ilp32
LDFLAGS:=$(CFLAGS) -Wl,--gc-sections -Wl,--strip-all

%.elf: %.o lib/libsim.o
	$(RVLD) $(LDFLAGS) -T linker.ld -o $@ $^
	$(RVSTRIP) -R .comment -R .riscv.attributes $@


%.mem: %.elf
	./make_mem.sh $<

%.o: %.c
	$(RVGCC) $(CFLAGS) -o $@ -c $^ 

%.o: %.s
	$(RVAS) $(CFLAGS) -o $@ -c $^ 
