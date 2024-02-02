;
; File generated by cc65 v 2.18 - N/A
;
	.fopt		compiler,"cc65 v 2.18 - N/A"
	.setcpu		"6502"
	.smart		on
	.autoimport	on
	.case		on
	.debuginfo	off
	.importzp	sp, sreg, regsave, regbank
	.importzp	tmp1, tmp2, tmp3, tmp4, ptr1, ptr2, ptr3, ptr4
	.macpack	longbranch
	.export		_str
	.export		_main
	.export		_RESETVEC

.segment	"DATA"

_str:
	.addr	L0001
.segment	"RESETVEC"
_RESETVEC:
	.word	$0F00
	.word	$0600
	.word	$0000
.segment	"DATA"

.segment	"RODATA"

L0001:
	.byte	$50,$72,$69,$6E,$74,$69,$6E,$67,$21,$0A,$00

; ---------------------------------------------------------------
; int __near__ main (void)
; ---------------------------------------------------------------

.segment	"CODE"

.proc	_main: near

.segment	"CODE"

;
; int i = 100 / 10;
;
	ldx     #$00
	lda     #$0A
	jsr     pushax
;
; while (1) {
;
	jmp     L0007
;
; if (i % 3 == 0 && i % 5 == 0) {
;
L0005:	ldy     #$01
	jsr     ldaxysp
	jsr     pushax
	ldx     #$00
	lda     #$03
	jsr     tosmodax
	cpx     #$00
	bne     L000B
	cmp     #$00
L000B:	jsr     booleq
	jeq     L000C
	ldy     #$01
	jsr     ldaxysp
	jsr     pushax
	ldx     #$00
	lda     #$05
	jsr     tosmodax
	cpx     #$00
	bne     L000D
	cmp     #$00
L000D:	jsr     booleq
	jne     L000A
L000C:	ldx     #$00
	lda     #$00
	jeq     L000E
L000A:	ldx     #$00
	lda     #$01
L000E:	jeq     L0009
;
; *(uint8_t *)(CHROUT) = 'F';
;
	ldx     #$00
	lda     #$46
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'i';
;
	ldx     #$00
	lda     #$69
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'z';
;
	ldx     #$00
	lda     #$7A
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'z';
;
	ldx     #$00
	lda     #$7A
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'B';
;
	ldx     #$00
	lda     #$42
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'u';
;
	ldx     #$00
	lda     #$75
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'z';
;
	ldx     #$00
	lda     #$7A
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'z';
;
	ldx     #$00
	lda     #$7A
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = '\n';
;
	ldx     #$00
	lda     #$0A
	sta     $FF00
;
; } else if  (i % 3 == 0) {
;
	jmp     L003E
L0009:	ldy     #$01
	jsr     ldaxysp
	jsr     pushax
	ldx     #$00
	lda     #$03
	jsr     tosmodax
	cpx     #$00
	bne     L002D
	cmp     #$00
L002D:	jsr     booleq
	jeq     L002B
;
; *(uint8_t *)(CHROUT) = 'F';
;
	ldx     #$00
	lda     #$46
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'i';
;
	ldx     #$00
	lda     #$69
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'z';
;
	ldx     #$00
	lda     #$7A
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'z';
;
	ldx     #$00
	lda     #$7A
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = '\n';
;
	ldx     #$00
	lda     #$0A
	sta     $FF00
;
; } else if (i % 5 == 0) {
;
	jmp     L003E
L002B:	ldy     #$01
	jsr     ldaxysp
	jsr     pushax
	ldx     #$00
	lda     #$05
	jsr     tosmodax
	cpx     #$00
	bne     L0040
	cmp     #$00
L0040:	jsr     booleq
	jeq     L003E
;
; *(uint8_t *)(CHROUT) = 'B';
;
	ldx     #$00
	lda     #$42
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'u';
;
	ldx     #$00
	lda     #$75
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'z';
;
	ldx     #$00
	lda     #$7A
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = 'z';
;
	ldx     #$00
	lda     #$7A
	sta     $FF00
;
; *(uint8_t *)(CHROUT) = '\n';
;
	ldx     #$00
	lda     #$0A
	sta     $FF00
;
; i++;
;
L003E:	ldy     #$01
	jsr     ldaxysp
	sta     regsave
	stx     regsave+1
	jsr     incax1
	ldy     #$00
	jsr     staxysp
	lda     regsave
	ldx     regsave+1
;
; while (1) {
;
L0007:	jmp     L0005
;
; for (;;);
;
L0006:	jmp     L0054
L0053:	jmp     L0006
L0054:	jmp     L0053
;
; }
;
	jsr     incsp2
	rts

.endproc

