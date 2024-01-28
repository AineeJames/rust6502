.setcpu "6502"

.segment "DATA"

hello: 
  .asciiz "Hello, World!"

index:
  .word $0000

.segment "CODE"

LDX #<hello ; load low addr of hello in x
LDY #>hello  ; load high addr of hello in a

LDA #$00    ; clear accum
JSR print   ; jump to print subroutine

JMP $0600   ; jump to org and rerun

print:
  LDA index
  CMP #$0D    ; Compare with the length of the string
  BEQ done    ; if eq you are done
  LDX index   ;
  LDA hello,X
  STA $6969
  INC index   ; inc index
  JMP print   ; restart the char print

done:
  RTS

.segment "RESETVEC"
  .word $0F00 ; NMI vector
  .word $0600 ; RESET vector
  .word $0000 ; IRQ vector
