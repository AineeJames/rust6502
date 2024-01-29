.setcpu "6502"

.define CHOUT $FF00

.segment "DATA"
hello: 
  .byte "Hello, World!", $0A, $00
index:
  .byte $00

.segment "CODE"
LDX #<hello   ; load low addr of hello in x
LDY #>hello   ; load high addr of hello in a

LDA #$00      ; clear accum
LDX #$00      ; store x w/ 0
STX index     ; clear index
JSR print     ; jump to print subroutine

JMP $0600     ; jump to org and rerun

print:
  LDA index
  CMP #$0F    ; Compare with the length of the string
  BEQ done    ; if eq you are done
  LDX index   ; laod index to x
  LDA hello,X ; load string addr index by index
  STA CHOUT   ; TODO: put char somewhere
  INC index   ; inc index
  JMP print   ; restart the char print

done:
  RTS

.segment "RESETVEC"
  .word $0F00 ; NMI vector
  .word $0600 ; RESET vector
  .word $0000 ; IRQ vector
