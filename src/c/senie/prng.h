#pragma once

#include "types.h"

struct senie_prng_state {
  u64 state[2];
};

void senie_prng_set_state(senie_prng_state* prng_state, u64 seed);
void senie_prng_copy(senie_prng_state* dest_prng_state, senie_prng_state* src_prng_state);

i32 senie_prng_i32_range(senie_prng_state* prng_state, i32 min, i32 max);

// returns a float in the range 0..1
f32 senie_prng_f32(senie_prng_state* prng_state);
f32 senie_prng_f32_range(senie_prng_state* prng_state, f32 min, f32 max);

// returns a number in the range -0.293498..0.293498
f32 senie_perlin(f32 x, f32 y, f32 z);
