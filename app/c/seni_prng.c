#include "seni_prng.h"

#include "seni_mathutil.h"

#define  RND_IMPLEMENTATION
#define RND_U32 u32
#define RND_U64 u64
#include "lib/rnd.h"

static unsigned char permutations[512] =
  {
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95,
    96, 53, 194, 233, 7, 225, 140, 36, 103, 30,
    69, 142, 8, 99, 37, 240, 21, 10, 23, 190,
    6, 148, 247, 120, 234, 75, 0, 26, 197, 62,
    94, 252, 219, 203, 117, 35, 11, 32, 57, 177,
    33, 88, 237, 149, 56, 87, 174, 20, 125, 136,
    171, 168, 68, 175, 74, 165, 71, 134, 139, 48,
    27, 166, 77, 146, 158, 231, 83, 111, 229, 122,
    60, 211, 133, 230, 220, 105, 92, 41, 55, 46,
    245, 40, 244, 102, 143, 54, 65, 25, 63, 161,
    1, 216, 80, 73, 209, 76, 132, 187, 208, 89,
    18, 169, 200, 196, 135, 130, 116, 188, 159, 86,
    164, 100, 109, 198, 173, 186, 3, 64, 52, 217,
    226, 250, 124, 123, 5, 202, 38, 147, 118, 126,
    255, 82, 85, 212, 207, 206, 59, 227, 47, 16,
    58, 17, 182, 189, 28, 42, 223, 183, 170, 213,
    119, 248, 152, 2, 44, 154, 163, 70, 221, 153,
    101, 155, 167, 43, 172, 9, 129, 22, 39, 253,
    19, 98, 108, 110, 79, 113, 224, 232, 178, 185,
    112, 104, 218, 246, 97, 228, 251, 34, 242, 193,
    238, 210, 144, 12, 191, 179, 162, 241, 81, 51,
    145, 235, 249, 14, 239, 107, 49, 192, 214, 31,
    181, 199, 106, 157, 184, 84, 204, 176, 115, 121,
    50, 45, 127, 4, 150, 254, 138, 236, 205, 93,
    222, 114, 67, 29, 24, 72, 243, 141, 128, 195,
    78, 66, 215, 61, 156, 180,

    151, 160, 137, 91, 90, 15, 131, 13, 201, 95,
    96, 53, 194, 233, 7, 225, 140, 36, 103, 30,
    69, 142, 8, 99, 37, 240, 21, 10, 23, 190,
    6, 148, 247, 120, 234, 75, 0, 26, 197, 62,
    94, 252, 219, 203, 117, 35, 11, 32, 57, 177,
    33, 88, 237, 149, 56, 87, 174, 20, 125, 136,
    171, 168, 68, 175, 74, 165, 71, 134, 139, 48,
    27, 166, 77, 146, 158, 231, 83, 111, 229, 122,
    60, 211, 133, 230, 220, 105, 92, 41, 55, 46,
    245, 40, 244, 102, 143, 54, 65, 25, 63, 161,
    1, 216, 80, 73, 209, 76, 132, 187, 208, 89,
    18, 169, 200, 196, 135, 130, 116, 188, 159, 86,
    164, 100, 109, 198, 173, 186, 3, 64, 52, 217,
    226, 250, 124, 123, 5, 202, 38, 147, 118, 126,
    255, 82, 85, 212, 207, 206, 59, 227, 47, 16,
    58, 17, 182, 189, 28, 42, 223, 183, 170, 213,
    119, 248, 152, 2, 44, 154, 163, 70, 221, 153,
    101, 155, 167, 43, 172, 9, 129, 22, 39, 253,
    19, 98, 108, 110, 79, 113, 224, 232, 178, 185,
    112, 104, 218, 246, 97, 228, 251, 34, 242, 193,
    238, 210, 144, 12, 191, 179, 162, 241, 81, 51,
    145, 235, 249, 14, 239, 107, 49, 192, 214, 31,
    181, 199, 106, 157, 184, 84, 204, 176, 115, 121,
    50, 45, 127, 4, 150, 254, 138, 236, 205, 93,
    222, 114, 67, 29, 24, 72, 243, 141, 128, 195,
    78, 66, 215, 61, 156, 180
  };


static f32 fade(f32 t)
{
  return t * t * t * (t * (t * 6.0f - 15.0f) + 10.0f);
}

static i32 fastfloor(f32 a)
{
	i32 ai = (i32) a;
	return (a < ai) ? ai-1 : ai;
}

static f32 grad_old(i32 hash, f32 x, f32 y, f32 z)
{
  i32 h = hash & 15;                      // CONVERT LO 4 BITS OF HASH CODE
  f32 u = h < 8 ? x : y;                 // INTO 12 GRADIENT DIRECTIONS.
  f32 v = h < 4 ? y : h == 12 || h == 14 ? x : z;
  return ((h & 1) == 0 ? u : -u) + ((h & 2) == 0 ? v : -v);
}

static f32 grad(i32 hash, f32 x, f32 y, f32 z)
{
   static f32 basis[12][4] =
   {
      {  1, 1, 0 },
      { -1, 1, 0 },
      {  1,-1, 0 },
      { -1,-1, 0 },
      {  1, 0, 1 },
      { -1, 0, 1 },
      {  1, 0,-1 },
      { -1, 0,-1 },
      {  0, 1, 1 },
      {  0,-1, 1 },
      {  0, 1,-1 },
      {  0,-1,-1 },
   };

   // perlin's gradient has 12 cases so some get used 1/16th of the time
   // and some 2/16ths. We reduce bias by changing those fractions
   // to 5/64ths and 6/64ths, and the same 4 cases get the extra weight.
   static unsigned char indices[64] =
   {
      0,1,2,3,4,5,6,7,8,9,10,11,
      0,9,1,11,
      0,1,2,3,4,5,6,7,8,9,10,11,
      0,1,2,3,4,5,6,7,8,9,10,11,
      0,1,2,3,4,5,6,7,8,9,10,11,
      0,1,2,3,4,5,6,7,8,9,10,11,
   };

   f32 *grad = basis[indices[hash & 15]];

   return grad[0]*x + grad[1]*y + grad[2]*z;
}

// A basic translation of Ken Perlin's Java
// reference implementation of improved noise (C) 2002

// returns a value in the range -1..1
f32 noise(f32 x_, f32 y_, f32 z_)
{
  i32 x_floor = fastfloor(x_);
  i32 y_floor = fastfloor(y_);
  i32 z_floor = fastfloor(z_);
  
  i32 X = x_floor & 255;
  i32 Y = y_floor & 255;
  i32 Z = z_floor & 255;

  f32 x = x_ - (f32)x_floor;
  f32 y = y_ - (f32)y_floor;
  f32 z = z_ - (f32)z_floor;

  f32 u = fade(x);
  f32 v = fade(y);
  f32 w = fade(z);

  i32 A = permutations[X] + Y;
  i32 AA = permutations[A] + Z;
  i32 AB = permutations[A + 1] + Z;
  i32 B = permutations[X + 1] + Y;
  i32 BA = permutations[B] + Z;
  i32 BB = permutations[B + 1] + Z;

  return lerp(w, lerp(v, lerp(u, grad(permutations[AA], x, y, z),        // AND ADD
                              grad(permutations[BA], x - 1, y, z)),      // BLENDED
                      lerp(u, grad(permutations[AB], x, y - 1, z),       // RESULTS
                           grad(permutations[BB], x - 1, y - 1, z))),    // FROM  8
              lerp(v, lerp(u, grad(permutations[AA + 1], x, y, z - 1),   // CORNERS
                           grad(permutations[BA + 1], x - 1, y, z - 1)), // OF CUBE
                   lerp(u, grad(permutations[AB + 1], x, y - 1, z - 1),
                        grad(permutations[BB + 1], x - 1, y - 1, z - 1))));
  
}

// ----------------------------------------------------------------------

void seni_prng_set_state(seni_prng_state *prng_state, u64 seed)
{
  rnd_pcg_seed((rnd_pcg_t *)prng_state, (u32)seed);
}

void seni_prng_copy(seni_prng_state *dest_prng_state, seni_prng_state *src_prng_state)
{
  dest_prng_state->state[0] = src_prng_state->state[0];
  dest_prng_state->state[1] = src_prng_state->state[1];
}

i32 seni_prng_i32_range(seni_prng_state* prng_state, i32 min, i32 max)
{
  i32 res = rnd_pcg_range((rnd_pcg_t *)prng_state, min, max);

  return res;
}

// 0..1
f32 seni_prng_f32(seni_prng_state* prng_state)
{
  f32 res = rnd_pcg_nextf((rnd_pcg_t *)prng_state);
  return res;
}

f32 seni_prng_f32_range(seni_prng_state* prng_state, f32 min, f32 max)
{
  f32 value = seni_prng_f32(prng_state);
  value = (value * (max - min)) + min;

  return value;
}

// some wrappers around the stb perlin noise implementation
// -1..1
f32 seni_perlin(f32 x, f32 y, f32 z)
{
  return noise(x, y, z);
}
