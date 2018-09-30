#pragma once

#include "types.h"

struct sen_prng_state {
  u64 state[2];
};

void sen_prng_set_state(sen_prng_state* prng_state, u64 seed);
void sen_prng_copy(sen_prng_state* dest_prng_state,
                   sen_prng_state* src_prng_state);

i32 sen_prng_i32_range(sen_prng_state* prng_state, i32 min, i32 max);

// returns a float in the range 0..1
f32 sen_prng_f32(sen_prng_state* prng_state);
f32 sen_prng_f32_range(sen_prng_state* prng_state, f32 min, f32 max);

// given a val, return a number that's within x percent of it
// e.g. given (50, 10, 0, 200) return a number that's within 50 +- 10%
// where the absolute range for 10% is found by max - min
// in this example 10% is 20, so the returned value will be 50 +- 20
f32 sen_prng_f32_around(sen_prng_state* prng_state, f32 val, f32 percent,
                        f32 min, f32 max);

// returns a number in the range -0.293498..0.293498
f32 sen_perlin(f32 x, f32 y, f32 z);
