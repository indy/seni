#include "seni_uv_mapper.h"

#include <stdlib.h>

i32 *num_uv_mappings = NULL;
seni_uv_mapping **g_brush_info = NULL;

f32 texture_dim = 1024.0f;

void make_uv(v2 *out, f32 in_u, f32 in_v)
{
  out->x = in_u / texture_dim;
  out->y = in_v / texture_dim;
}

void allocate_uv_mapping(seni_brush_type type, i32 sub_type,
                         i32 min_x, i32 min_y, i32 max_x, i32 max_y,
                         f32 width_scale)
{
  seni_uv_mapping *m = &(g_brush_info[type][sub_type]);

  m->map = malloc(4 * sizeof(v2));

  m->width_scale = width_scale;

  make_uv(&(m->map[0]), (f32)max_x, (f32)min_y);
  make_uv(&(m->map[1]), (f32)max_x, (f32)max_y);
  make_uv(&(m->map[2]), (f32)min_x, (f32)min_y);
  make_uv(&(m->map[3]), (f32)min_x, (f32)max_y);
}

void init_uv_mapper()
{
  g_brush_info = (seni_uv_mapping **)malloc(NUM_BRUSHES * sizeof (seni_uv_mapping *));

  num_uv_mappings = (i32 *)malloc(NUM_BRUSHES * sizeof(i32));
  num_uv_mappings[BRUSH_FLAT] = 1;
  num_uv_mappings[BRUSH_A] = 1;
  num_uv_mappings[BRUSH_B] = 6;
  num_uv_mappings[BRUSH_C] = 9;
  num_uv_mappings[BRUSH_D] = 1;
  num_uv_mappings[BRUSH_E] = 1;
  num_uv_mappings[BRUSH_F] = 1;
  num_uv_mappings[BRUSH_G] = 2;

  for (i32 i=BRUSH_FLAT; i < NUM_BRUSHES; i++) {
    g_brush_info[i] = (seni_uv_mapping *)malloc(num_uv_mappings[i] * sizeof(seni_uv_mapping));
  }

  // BRUSH_FLAT
  allocate_uv_mapping(BRUSH_FLAT, 0, 1, 1, 2, 2, 1.0f);
  // BRUSH_A
  allocate_uv_mapping(BRUSH_A, 0, 0, 781, 976, 1023, 1.2f);
  // BRUSH_B
  allocate_uv_mapping(BRUSH_B, 0, 11, 644, 490, 782, 1.4f);
  allocate_uv_mapping(BRUSH_B, 1, 521, 621, 1023, 783, 1.1f);
  allocate_uv_mapping(BRUSH_B, 2, 340, 419, 666, 508, 1.2f);
  allocate_uv_mapping(BRUSH_B, 3, 326, 519, 659, 608, 1.2f);
  allocate_uv_mapping(BRUSH_B, 4, 680, 419, 1020, 507, 1.1f);
  allocate_uv_mapping(BRUSH_B, 5, 677, 519, 1003, 607, 1.1f);
  // BRUSH_C
  allocate_uv_mapping(BRUSH_C, 0, 0, 7, 324, 43, 1.2f);
  allocate_uv_mapping(BRUSH_C, 1, 0, 45, 319, 114, 1.3f);
  allocate_uv_mapping(BRUSH_C, 2, 0, 118, 328, 180, 1.1f);
  allocate_uv_mapping(BRUSH_C, 3, 0, 186, 319, 267, 1.2f);
  allocate_uv_mapping(BRUSH_C, 4, 0, 271, 315, 334, 1.4f);
  allocate_uv_mapping(BRUSH_C, 5, 0, 339, 330, 394, 1.1f);
  allocate_uv_mapping(BRUSH_C, 6, 0, 398, 331, 473, 1.2f);
  allocate_uv_mapping(BRUSH_C, 7, 0, 478, 321, 548, 1.1f);
  allocate_uv_mapping(BRUSH_C, 8, 0, 556, 326, 618, 1.1f);
  // BRUSH_D
  allocate_uv_mapping(BRUSH_D, 0, 333, 165, 734, 336, 1.3f);
  // BRUSH_E
  allocate_uv_mapping(BRUSH_E, 0, 737, 183, 1018, 397, 1.3f);
  // BRUSH_F
  allocate_uv_mapping(BRUSH_F, 0, 717, 2, 1023, 163, 1.1f);
  // BRUSH_G
  allocate_uv_mapping(BRUSH_G, 0, 329, 0, 652, 64, 1.2f);
  allocate_uv_mapping(BRUSH_G, 1, 345, 75, 686, 140, 1.0f);
}


void free_uv_mapping(seni_brush_type type)
{
  seni_uv_mapping *m = g_brush_info[type];
  i32 num = num_uv_mappings[type];
  
  for (i32 i = 0; i < num; i++) {
    seni_uv_mapping *p = &(m[i]);
    free(p->map);
  }
  
  free(m);
}

void free_uv_mapper()
{
  free_uv_mapping(BRUSH_FLAT);
  free_uv_mapping(BRUSH_A);
  free_uv_mapping(BRUSH_B);
  free_uv_mapping(BRUSH_C);
  free_uv_mapping(BRUSH_D);
  free_uv_mapping(BRUSH_E);
  free_uv_mapping(BRUSH_F);
  free_uv_mapping(BRUSH_G);

  free(g_brush_info);
  free(num_uv_mappings);
}

seni_uv_mapping *get_uv_mapping(seni_brush_type type, i32 sub_type)
{
  if(num_uv_mappings[type] <= sub_type) {
    return NULL;
  }
  return &(g_brush_info[type][sub_type]);
}
