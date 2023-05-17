
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
        mov rax, 7
test rax, 3
mov rax, 7
mov rbx, 3
cmove rax, rbx
        ret
