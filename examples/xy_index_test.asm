LDX #$69   ; x = 69
STX $1234  ; mem[1234] = 69
LDY $1234  ; y = mem[1234] = 69
LDX #$68   ; x = 68
STY $01,X  ; mem[1 + x] = y

LDX #$70   ; x = 70
CPX $1234  ; x >= mem[1234] -> set(C)

CPY $1234  ; y == mem[1234] -> set(Z)

LDX #$03   ; x = 3
STX $1234  ; mem[1234] = 3 
DEC $1234  ; mem[1234] = 2
DEC $1234  ; mem[1234] = 1
DEC $1234  ; mem[1234] = 0

DEX        ; x = 2
DEY        ; y = 68
