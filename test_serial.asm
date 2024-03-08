bits 32

section .text
global start

start:
  mov dx, 0x3f8
  mov ax, 0x4141
  out dx, ax
  in ax, dx
  out dx, ax
  hlt
  ; mov al, 0x41
;   out dx, al
;   lea rsi, [rel msg]
;   .next_char:
;     lodsb             ; load next byte from si into al
;     or al, al         ; if al == 0 then we are done
;     jz .done
;     call putchar
;     jmp .next_char

; .done:
;   hlt

; putchar:
;   mov dx, 0x3f8
;   .wait:
;       in al, dx
;       test al, 0x20
;       jz .wait
;   mov al, [rsi - 1]   ; [rsi-1], rsi already incremented by lodsb
;   out dx, al
;   ret

; msg db 'HELLO WORLD, LET SPREAD FREEDOM TO RIOT', 0
