
        section .text
        global our_code_starts_here
        extern snek_error
        extern snek_print
        error_handling_starts_here:
        invalid_argument:
          mov rdi, 99
          jmp throw_error
        overflow:
          mov rdi, 101
          jmp throw_error
        throw_error:
          push rsp
          call snek_error
          ret
        function_defination_starts_here:
        print:
          mov rdi, [rsp + 8]
          push rsp
          call snek_print
          pop rsp
          ret
        
        our_code_starts_here:
          mov r15,rsi
          mov [r15 - 0], rax
mov rax, 2
mov [r15 - 8], rax
mov rax, 4
mov [r15 - 16], rax
mov rax, 6
mov [r15 - 24], rax
mov rax, r15
add rax, 1
add r15, 32
and rax, 3
cmp rax, 3
mov rax, 3
mov rbx, 7
cmove rax, rbx
          ret

