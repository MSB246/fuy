debug:
	cargo run $(SOURCE) $(OUTPUT)

release:
	cargo run --release $(SOURCE) $(OUTPUT)

build:
	nasm -f elf64 example.asm
	ld example.o

run:
	./example