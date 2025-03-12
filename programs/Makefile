RVGCC:=riscv64-unknown-elf-gcc
RVLD:=$(RVGCC)
#RVLD:=riscv32-unknown-elf-ld
RVAS:=riscv64-unknown-elf-as
RVSTRIP:=riscv64-unknown-elf-strip
CFLAGS:=-fpic -march=rv32i -mabi=ilp32
LIBS:=-lgcc
LDFLAGS:=$(CFLAGS) -Wl,--gc-sections -nostdlib  -Wl,--strip-all
TARGETS:=call_subroutine.mem simple_test_mem.mem test_mem.mem

all: $(TARGETS)

clean: 
	rm -f $(TARGETS)
	rm -f lib/libstartup.o

%.elf: %.o lib/libstartup.o
	$(RVLD) $(LDFLAGS) -T linker.ld -o $@ $^ $(LIBS)
	$(RVSTRIP) -R .comment -R .riscv.attributes $@


%.mem: %.elf
	./make_mem.sh $<

%.o: %.c
	$(RVGCC) $(CFLAGS) -o $@ -c $^ 

%.o: %.s
	$(RVAS) $(CFLAGS) -o $@ -c $^ 

