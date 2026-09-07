bits 64
global _start

section .text
_start:
    hlt             ; trigger a #GP
