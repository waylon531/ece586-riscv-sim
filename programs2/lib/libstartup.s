.global _start
.extern main, exit
.section .text.prologue
_start:
        call main
        call _exit
.global _exit
_exit:
        add ra,zero,zero
        ret
