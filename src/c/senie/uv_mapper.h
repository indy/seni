#pragma once

#include "types.h"

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
} senie_brush_type;

struct senie_uv_mapping {
  f32  width_scale;
  f32* map; // array of 8 (4 pairs of xy)
};

void uv_mapper_subsystem_startup();
void uv_mapper_subsystem_shutdown();

void              make_uv(f32* outx, f32* ouyt, f32 in_u, f32 in_v);
senie_uv_mapping* get_uv_mapping(senie_brush_type type, i32 sub_type, bool wrap_sub_type);
