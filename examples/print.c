#include <stdio.h>
#include <stdint.h>
#include <string.h>

#define CHROUT 0xFF00
#define __BSS_RUN__ 0

const char* str = "Printing!\n";

int main() {

  int i = 100 / 10;
  while (1) {
    if (i % 3 == 0 && i % 5 == 0) {

      *(uint8_t *)(CHROUT) = 'F';
      *(uint8_t *)(CHROUT) = 'i';
      *(uint8_t *)(CHROUT) = 'z';
      *(uint8_t *)(CHROUT) = 'z';
      *(uint8_t *)(CHROUT) = 'B';
      *(uint8_t *)(CHROUT) = 'u';
      *(uint8_t *)(CHROUT) = 'z';
      *(uint8_t *)(CHROUT) = 'z';
      *(uint8_t *)(CHROUT) = '\n';
    } else if  (i % 3 == 0) {
      *(uint8_t *)(CHROUT) = 'F';
      *(uint8_t *)(CHROUT) = 'i';
      *(uint8_t *)(CHROUT) = 'z';
      *(uint8_t *)(CHROUT) = 'z';
      *(uint8_t *)(CHROUT) = '\n';
    } else if (i % 5 == 0) {

      *(uint8_t *)(CHROUT) = 'B';
      *(uint8_t *)(CHROUT) = 'u';
      *(uint8_t *)(CHROUT) = 'z';
      *(uint8_t *)(CHROUT) = 'z';
      *(uint8_t *)(CHROUT) = '\n';
    }

    i++;
  }
  for (;;);
}

#pragma data-name(push, "RESETVEC")
uint16_t RESETVEC[3] = {
    0x0F00, // NMI vector
    0x0600, // RESET vector
    0x0000  // IRQ vector
};
#pragma data-name(pop)


