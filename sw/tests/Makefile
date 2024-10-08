.PHONY: all clean

SRC_DIR=src

SOURCES=$(notdir $(wildcard $(SRC_DIR)/*.c))
TESTS=$(basename $(SOURCES))
ELF=$(addprefix $(BUILD_DIR)/,$(TESTS))
HEX=$(addsuffix .hex,$(ELF))

ELF2HEX=../../tools/elf2hex
VLIW_COMP_DRIVER=../../compiler/driver.py
RISCVCC32=riscv64-elf-gcc -march=rv32i -mabi=ilp32 -fno-builtin -static -nostdlib -nostartfiles -mcmodel=medany -Wno-implicit-function-declaration

all: $(HEX)

$(ELF2HEX)/elf2hex:
	$(MAKE) -C $(ELF2HEX)

$(BUILD_DIR)/init.o: init.S
	$(RISCVCC32) -c $^ -o $@

$(BUILD_DIR)/mmio.o: ../common/mmio.c
	$(RISCVCC32) -c $^ -o $@


$(BUILD_DIR)/%.hex: $(SRC_DIR)/%.c $(BUILD_DIR)/init.o $(BUILD_DIR)/mmio.o tests.ld
	mkdir -p $(BUILD_DIR)
	$(RISCVCC32) -O2 -I../common/ -c $(SRC_DIR)/$*.c -o $(BUILD_DIR)/test.o
	$(RISCVCC32) -o $(BUILD_DIR)/$* -Ttests.ld $(BUILD_DIR)/test.o $(BUILD_DIR)/init.o $(BUILD_DIR)/mmio.o
	@rm $(BUILD_DIR)/test.o
#ifeq ($(CORE),VLIW)
	$(VLIW_COMP_DRIVER) $(BUILD_DIR)/$* $(BUILD_DIR)/$*.hex > $(BUILD_DIR)/$*_pp.asm
#else
#	$(ELF2HEX)/elf2hex $(BUILD_DIR)/$* 0 16G $(BUILD_DIR)/$*.hex
#endif

