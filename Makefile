debug:
	cargo run $(SOURCE) $(OUTPUT)

release:
	cargo run --release $(SOURCE) $(OUTPUT)

build:
	nasm -f elf64 example.asm -o example.o
	ld example.o -o example

run:
	./example