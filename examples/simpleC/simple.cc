#include <stdint.h>
#include <stdio.h>

#include "rusty_radamsa.h"

void *radamsa_handle = NULL;

int main() {
    radamsa_handle = rusty_radamsa_init();
    int seed;
    const char in_buff[72] = "ABCDE\nKLMNOPQRSTUV\nZYX\nfeklafnewlka\nkelwflknewfw\n123214324\nhello world\n";
    char out_buff[80] = {0};

    for ( seed = 0; seed < 10; seed++) {
        rusty_radamsa(radamsa_handle, (const uint8_t*)in_buff, sizeof(in_buff), (uint8_t*)out_buff, sizeof(out_buff), seed);
        printf("%s\n", out_buff);
    }
   return 0;
}