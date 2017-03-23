#ifndef SENI_TYPES_H
#define SENI_TYPES_H

#include <stdint.h>

#include "uthash/uthash.h"

typedef int8_t   i8;
typedef int16_t  i16;
typedef int32_t  i32;
typedef int64_t  i64;

typedef uint8_t  u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

typedef float    f32;
typedef double   f64;

#define SENI_SCALAR
typedef f32 scalar;

// #include "gl-matrix/gl-matrix.h"

typedef struct
{
  scalar x, y;
} v2;

typedef struct
{
  int id;                    /* key */
  f32 ff;
  char name[10];
  UT_hash_handle hh;         /* makes this structure hashable */
} my_struct;

#endif  /* SENI_TYPES_H */
