#pragma once

#include "seni_types.h"

typedef struct {
  u64 state;
  u64 inc;
} seni_prng_state;


void seni_prng_set_state(seni_prng_state *prng_state, u64 seed);

// returns a u32 in the range 0..max
u32 seni_prng_u32(seni_prng_state* prng_state, u32 max);

// returns a float in the range 0..1
f32 seni_prng_f32(seni_prng_state* prng_state);
f32 seni_prng_f32_range(seni_prng_state* prng_state, f32 min, f32 max);

// returns a number in the range -0.293498..0.293498
f32 seni_perlin(f32 x, f32 y, f32 z);

