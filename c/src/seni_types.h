#ifndef SENI_TYPES_H
#define SENI_TYPES_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

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

#endif  /* SENI_TYPES_H */
