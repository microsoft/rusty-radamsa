#include <stdint.h>
#include <stdio.h>

#include "rusty_radamsa.h"

void *radamsa_handle = NULL;

extern "C" size_t LLVMFuzzerMutate(uint8_t *, size_t, size_t);

extern "C" size_t LLVMFuzzerCustomMutator(uint8_t *Data, size_t Size,
                                          size_t MaxSize, unsigned int Seed)
{
    if (Size == 0) {
        return LLVMFuzzerMutate(Data, Size, MaxSize);
    }
    size_t NewSize = rusty_radamsa(radamsa_handle, Data, Size, Data, MaxSize, Seed);
    
    return NewSize;
}

extern "C" int LLVMFuzzerInitialize(int *argc, char ***argv) {
    radamsa_handle = rusty_radamsa_init();
    rusty_radamsa_set_mutator(radamsa_handle, (const uint8_t*)"default");
    return 0;
}

extern "C" int LLVMFuzzerTestOneInput(const uint8_t *Data, size_t Size) {
  if (Size < 6) return 0;

  if (Data[0] == 'b')
    if (Data[1] == 'o')
      if (Data[2] == 'o')
        if (Data[3] == 'm')
          if (Data[4] == 'm')
            if (Data[5] == 'm')
              int x = *(int *)0x4141;

  return 0;
}