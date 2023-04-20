ROOT = $(shell git rev-parse --show-toplevel)
TARGET = hvc-loop

all: build

build: elf dump bin

elf:
	@cargo build --release

dump: elf
	@cargo objdump --release -- --disassemble --no-show-raw-insn > $(TARGET).dump

bin: elf
	@cargo objcopy --release -- -O binary > $(TARGET).bin

clean:
	@cargo clean
