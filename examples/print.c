#include <stdio.h>
#include <stdint.h>
#include <string.h>

#define CHROUT 0xFF00
#define __BSS_RUN__ 0

const char* str = "Printing!\n";

int main() {
    *(uint8_t *)(CHROUT) = 'H';
    *(uint8_t *)(CHROUT) = 'e';
    *(uint8_t *)(CHROUT) = 'l';
    *(uint8_t *)(CHROUT) = 'l';
    *(uint8_t *)(CHROUT) = 'o';
    *(uint8_t *)(CHROUT) = '\n';
    for (;;);
}

#pragma data-name(push, "RESETVEC")
uint16_t RESETVEC[3] = {
    0x0F00, // NMI vector
    0x0600, // RESET vector
    0x0000  // IRQ vector
};
#pragma data-name(pop)


