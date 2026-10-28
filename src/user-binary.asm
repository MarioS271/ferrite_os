bits 64
global _start

section .text
_start:
    mov rax, 0x293a
    push rax

    mov eax, 1
    mov rdi, rsp
    mov esi, 2
    syscall

    ; invalid syscall number
    ; logs syscall fail with error code 1 (InvalidSyscall)
    mov eax, 42
    syscall

    ; debug write
    mov eax, 1
    lea rdi, [rel hello_world]
    mov esi, hello_world_len
    syscall

    ; debug write (kernel addr)
    ; logs syscall fail with error code 4 (BadAddress)
    mov eax, 1
    mov rdi, 0xFFFF900000000000
    mov esi, hello_world_len
    syscall

    ; debug write (invalid absolute addr)
    mov eax, 1
    mov rdi, 0x1000
    mov esi, hello_world_len
    syscall

    ; syscall with current byte as output
    mov eax, 1
    mov rdi, $
    mov esi, 1
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
