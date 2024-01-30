.setcpu "6502"

.define CHOUT $FF00

.segment "DATA"
index:
  .byte $00

.segment "CODE"

ldy #0
loop:
  tya
  iny
  tya
  jsr PrDec
  lda #$0a ; newline
  sta CHOUT
  jmp loop

; On entry, A=value to print
; On exit,  A corrupted
PrDec:
  ldx #$FF
  sec                ; Prepare for subtraction
PrDec100:
  inx
  sbc #100
  bcs PrDec100       ; Count how many 100s
  adc #100
  jsr PrDecDigit     ; Print the 100s
  ldx #$FF
  sec                ; Prepare for subtraction
PrDec10:
  inx
  sbc #10
  bcs PrDec10        ; Count how many 10s
  adc #10
  jsr PrDecDigit     ; Print the 10s
  tax                ; Pass 1s into X
PrDecDigit:
  pha
  txa                ; Save A, pass digit to A
  clc
  adc #$30
  sta CHOUT
  pla
  rts                ; Restore A and return

.segment "RESETVEC"
  .word $0F00 ; NMI vector
  .word $0600 ; RESET vector
  .word $0000 ; IRQ vector
