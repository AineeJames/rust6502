.setcpu "6502"

.define CHOUT $FF00

.segment "DATA"

; 1
; 2
; 3 Fizz
; 4
; 5 Buzz
; ...
; 15 Fizz Buzz

fizzstr: 
  .asciiz "Fizz"
buzzstr: 
  .asciiz "Buzz"
space:
  .asciiz " "
newline:
  .byte $0A, $00
counter:
  .byte $01

.segment "CODE"

start:

  jsr printcounter
  ldx #<space
  jsr print
  ldx #<fizzstr
  jsr print
  ldx #<space
  jsr print
  ldx #<buzzstr
  jsr print
  ldx #<newline
  jsr print

  ; inc counter and break out of loop if at max val
  inc counter
  lda counter
  cmp $FF
  beq endloop

  jmp start

endloop:
  jmp endloop


;n = 12
;>= 1 ? yes
;>= 10 ? yes
;>= 100 ? no
;/ 10 = 1
;print 1 
;n - 10
;2 
;>= 1 ? yes
;2/ 1 = 2
;print 2
;done


printcounter:
  lda counter
  clc
  jsr print_100s_place
  ;jsr print_10s_place
  ;jsr print_1s_place

  adc #$30 ; numb as char
  sta CHOUT

endprintcounter:
  rts

print_100s_place:
  ;push x
  TXS

  cmp #100 
  bne print_100s_end
  ldx #0
  let_code_flow:  
  sbc #100
  inx
  cmp #100
  beq let_code_flow

  ;Get x to accumulator
  TXA 
  adc #$30
  sta CHOUT
  
  ;popx
  TSX

print_100s_end:
  rts


print:
  lda $00,X
  cmp #0
  beq endprint
  sta CHOUT
  inx
  jmp print
endprint:
  rts

.segment "RESETVEC"
  .word $0F00 ; NMI vector
  .word $0600 ; RESET vector
  .word $0000 ; IRQ vector
