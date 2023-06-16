#include <stdint.h>
extern "C" void *rusty_radamsa_init();
//rusty_radamsa_set_mutator(ctx: *mut Radamsa, config: *const i8) 
extern "C" void rusty_radamsa_set_mutator(void *, const uint8_t *);
// rusty_radamsa(ctx: *mut Radamsa, data: *const u8, size: usize, out: *mut u8, max_size: usize, seed: u64) -> usize
extern "C" size_t rusty_radamsa(void *, const uint8_t *, const size_t, uint8_t *, const size_t, const size_t);