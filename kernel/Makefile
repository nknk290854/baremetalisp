# Default to the RPi4
BSP=raspi3
ifndef BSP
	BSP = raspi4
endif

# BSP-specific arguments
ifeq ($(BSP),raspi3)
	RUSTC_MISC_ARGS = -C target-cpu=cortex-a53
else ifeq ($(BSP),raspi4)
	RUSTC_MISC_ARGS = -C target-cpu=cortex-a72
endif

#ASM_FILE=asm/aarch64.S
#ASM_OBJ=aarch64.o

#ASM_FILE=asm/boot.S
#ASM_OBJ=boot.o
ASM_FILE=asm/pine64.S
ASM_OBJ=pin64.o

C_FILE=c/mmu_a.c
C_OBJ=mmu_a.o

TARGET=aarch64-unknown-none

RUSTLIB=target/$(TARGET)/release/libbaremetalisp.a
RUSTFLAGS=$(RUSTC_MISC_ARGS)

CC=aarch64-linux-gnu-gcc
LD=aarch64-linux-gnu-ld

all: kernel8.img

$(ASM_OBJ): $(ASM_FILE)
	$(CC) -c $(ASM_FILE) -o $(ASM_OBJ) -D$(BSP)

$(C_OBJ): $(C_FILE)
	$(CC) -c $(C_FILE) -o $(C_OBJ) -D$(BSP)

$(RUSTLIB): FORCE
	RUSTFLAGS="$(RUSTFLAGS)" cargo xrustc --features $(BSP) --target $(TARGET) --release

doc:
	cargo xdoc --target=$(TARGET) --features $(BSP) --document-private-items

baremetalisp: $(RUSTLIB) $(MMU_OBJ) $(ASM_OBJ) $(C_OBJ)
	$(LD) -T link.ld -o baremetalisp $(ASM_OBJ) $(C_OBJ) $(RUSTLIB)

kernel8.img: baremetalisp
	aarch64-linux-gnu-objcopy -O binary baremetalisp kernel8.img

AARCH64_START_ADDRESS=0x40080000
SUNXI_FEL=sunxi-fel

boot64:
	sudo ${SUNXI_FEL} spl sunxi-a64-spl32-ddr3.bin
	sudo ${SUNXI_FEL} write ${AARCH64_START_ADDRESS} kernel8.img
	sudo ${SUNXI_FEL} reset64 ${AARCH64_START_ADDRESS}

rmobj: FORCE
	rm -f baremetalisp kernel8.img *.o

clean:
	cargo clean
	rm -f baremetalisp kernel8.img *.o

FORCE:

build:
	docker exec -w '/hostdir/baremetalisp/kernel' 8919ef /usr/bin/zsh -c make

