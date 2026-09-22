bits 64
default rel
global _start

section .text
_start:
    ; invalid syscall number, will get logged
    mov eax, 42
    syscall

    ; debug write
    mov eax, 1
    lea rdi, [hello_world]
    mov esi, hello_world_len
    syscall

    ; exit program, will panic the kernel
    mov eax, 0
    mov edi, 0
    syscall

_halt:
    jmp _halt

section .rodata
hello_world: db "Hello, World!"
hello_world_len: equ $ - hello_world
