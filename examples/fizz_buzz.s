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
  .byte $01 ; start at 11 for testing

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
  ;if a > 100 -100
  ;need to subtract to just tens here
  clc
  jsr print_10s_place
  ;if a > 10 -10
  
  lda counter
  clc
  jsr print_1s_place

  ;adc #$30 ; numb as char
  ;sta CHOUT

endprintcounter:
  rts

print_100s_place:
  ;push x
  TXS

  cmp #100 
  beq print_100s_end
  ldx #0
  let_code_flow:  
  sbc #100
  bcs stop_code_flow
  bne not_zero
  inx
  jmp stop_code_flow
  not_zero:
  inx
  cmp #100
  bne let_code_flow

stop_code_flow:

  ;Get x to accumulator
  TXA 
  clc
  adc #$30
  sta CHOUT
  
  TXA 
  cmp #0
  bne skip_counter_assignment

  lda counter
  jmp print_100s_end

  skip_counter_assignment:

  sub_100s_loop:
  sbc #100
  dex
  beq print_100s_end
  jmp sub_100s_loop



  jmp print_100s_end
  ;popx

print_100s_end:
  TSX
  rts

print_10s_place:
  txs
  cmp #10
  bcc stop_10s_flow
  ldx #0
  let_10s_flow:
  sbc #10
  bcs stop_10s_flow
  bne not_zero_10s
  jmp stop_10s_flow
  not_zero_10s:
  inx
  cmp #10
  bne let_10s_flow

stop_10s_flow:
  TXA
  clc
  adc #$30
  sta CHOUT
  
  jmp print_10s_end



print_10s_end:
  TSX
  rts

print_1s_place:
  txs
  ;acc has num to print
  cmp #10
  bcc stop_1s_flow

  sub_10s_loop:
  sbc #10
  cmp #10
  bcc stop_1s_flow
  jmp sub_10s_loop

stop_1s_flow:
  clc
  adc #$30
  sta CHOUT
  jmp print_1s_end

print_1s_end:
  TSX
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
