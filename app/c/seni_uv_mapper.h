#ifndef SENI_UV_MAPPER_H
#define SENI_UV_MAPPER_H

#include "seni_types.h"

typedef enum {
  BRUSH_FLAT = 0,
  BRUSH_A,
  BRUSH_B,
  BRUSH_C,
  BRUSH_D,
  BRUSH_E,
  BRUSH_F,
  BRUSH_G,
  NUM_BRUSHES
} seni_brush_type;

typedef struct seni_uv_mapping {
  f32 width_scale;
  f32 *map;                  // array of 8 (4 pairs of xy)
} seni_uv_mapping;

void init_uv_mapper();
void free_uv_mapper();


void make_uv(f32 *outx, f32 *ouyt, f32 in_u, f32 in_v);
seni_uv_mapping *get_uv_mapping(seni_brush_type type, i32 sub_type, bool wrap_sub_type);

#endif
