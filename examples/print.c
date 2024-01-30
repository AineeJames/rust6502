#include <stdio.h>
#include <stdint.h>
#include <string.h>

int val = 43;
const char* str = "Hello, World!\n";



int main() {
    int i = 0;
    while (str[i] != 0) {
        uintptr_t physical_address = 0xFF00;
        volatile uint16_t *memory_location = (uint16_t *)physical_address;
        *memory_location = str[i];

        i++;
    }
    for (;;) {}
}

#pragma data-name(push, "RESETVEC")
uint16_t RESETVEC[3] = {
    0x0F00, // NMI vector
    0x0600, // RESET vector
    0x0000  // IRQ vector
};
#pragma data-name(pop)


