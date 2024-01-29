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
  adc #$30 ; numb as char
  sta CHOUT

endprintcounter:
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
