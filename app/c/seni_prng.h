#pragma once

#include "seni_types.h"

struct seni_prng_state {
  u64 state[2];
};

void seni_prng_set_state(seni_prng_state *prng_state, u64 seed);
void seni_prng_copy(seni_prng_state *dest_prng_state, seni_prng_state *src_prng_state);

i32 seni_prng_i32_range(seni_prng_state* prng_state, i32 min, i32 max);

// returns a float in the range 0..1
f32 seni_prng_f32(seni_prng_state* prng_state);
f32 seni_prng_f32_range(seni_prng_state* prng_state, f32 min, f32 max);

// returns a number in the range -0.293498..0.293498
f32 seni_perlin(f32 x, f32 y, f32 z);
