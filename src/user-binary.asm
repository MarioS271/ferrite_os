bits 64
global _start

section .text
_start:
    mov eax, 42
    syscall
    mov eax, 69
    syscall
    mov eax, 0
    mov edi, 0
    syscall
_halt:
    jmp _halt
