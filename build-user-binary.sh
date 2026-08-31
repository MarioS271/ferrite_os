#!/usr/bin/env bash
nasm -f elf64 src/user-binary.asm -o build/user-binary.o
ld -static -nostdlib build/user-binary.o -o src/kernel/resources/user-binary
rm build/user-binary.o
