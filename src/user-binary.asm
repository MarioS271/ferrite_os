bits 64
global _start

section .text
_start:
    hlt             ; trigger a #GP
    mov rax, 60     ; syscall code for exit
    xor rdi, rdi    ; exit code 0
    syscall
