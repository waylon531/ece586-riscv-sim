RVGCC:=riscv64-unknown-elf-gcc
RVLD:=$(RVGCC)
#RVLD:=riscv32-unknown-elf-ld
RVAS:=riscv64-unknown-elf-as
RVSTRIP:=riscv64-unknown-elf-strip
ASFLAGS:=-fpic -march=rv32im -mabi=ilp32
CFLAGS:=$(ASFLAGS) -I./lib/ #-O2 Extra speed :)
#CFLAGS+=-O2 Extra speed :)
LIBS:=-lgcc
LDFLAGS:=$(ASFLAGS) -Wl,--gc-sections -nostdlib  -Wl,--strip-all
TARGETS:=test/call_subroutine.mem test/simple_test_mem.mem test/test_mem.mem test/lbu_test.mem 
TARGETS+=jpg_decode/jpg_decode.mem checker.mem echo.mem loop_print.mem

all: $(TARGETS)

clean: 
	rm -f $(TARGETS)
	rm -f lib/*.o

%.elf: %.o lib/libstartup.o
	$(RVLD) $(LDFLAGS) -T linker.ld -o $@ $^ $(LIBS)
	$(RVSTRIP) -R .comment -R .riscv.attributes $@


%.mem: %.elf
	./make_mem.sh $<

%.o: %.c
	$(RVGCC) $(CFLAGS) -o $@ -c $^ 

%.o: %.s
	$(RVAS) $(ASFLAGS) -o $@ -c $^ 

jpg_decode/jpg_decode.elf: lib/libstartup.o lib/libremu.o jpg_decode/jpg_decode.o jpg_decode/need_bytes.o picojpeg/picojpeg.o 
	$(RVLD) $(LDFLAGS) -T linker.ld -o $@ $^ $(LIBS)
	$(RVSTRIP) -R .comment -R .riscv.attributes $@

