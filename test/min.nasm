global _start

section .text

_start:
    ; exit system call (Linux / x86-64)
    mov rax, 60  ; system call id
    mov rdi, 1  ; return code
    syscall  ; system call op code

