.setcpu "6502"

.segment "CODE"
LDX #$70    ; x = 70
CPX $1234   ; x >= mem[1234] -> set(C)

CPY $1234   ; y == mem[1234] -> set(Z)

LDX #$03    ; x = 3
STX $1234   ; mem[1234] = 3 
DEC $1234   ; mem[1234] = 2
DEC $1234   ; mem[1234] = 1
DEC $1234   ; mem[1234] = 0

DEX         ; x = 2
DEY         ; y = 68

LDX #$00    ; x = 00
STX $2000   ; mem[2000] = 00
LDX #$06    ; x = 06
STX $2001   ; mem[2001] = 06
JMP ($2000) ; jump to org and rerun

.segment "RESETVEC"
  .word $0F00 ; NMI vector
  .word $0600 ; RESET vector
  .word $0000 ; IRQ vector
  
