#!/bin/bash

cc65 print.c -o print.s -v -T --cpu 6502
grep -v "	.forceimport	__STARTUP__" print.s > tmpfile && mv tmpfile print.s
ca65 -o print.o print.s -t none
ld65 print.o -o print -C bios.cfg --lib /opt/homebrew/Cellar/cc65/2.19/share/cc65/lib/none.lib
