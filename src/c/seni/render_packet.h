#pragma once

#include "types.h"

struct seni_render_packet {
  // number of vertices actually in the render_packet
  int num_vertices;

  f32* vbuf; // max_vertices * vbuf_element_size * sizeof(f32)
  f32* cbuf; // max_vertices * cbuf_element_size * sizeof(f32)
  f32* tbuf; // max_vertices * tbuf_element_size * sizeof(f32)

  struct seni_render_packet* prev;
  struct seni_render_packet* next;
};

struct seni_render_data {
  // max number of vertices that can fit in a single render_packet
  i32 max_vertices;

  i32 vbuf_element_size; // 2
  i32 cbuf_element_size; // 4
  i32 tbuf_element_size; // 2

  // head of linked list of render packets
  seni_render_packet* render_packets;

  i32 num_render_packets;

  // the current render packet that should be filled in
  seni_render_packet* current_render_packet;
};

seni_render_data* render_data_allocate(i32 max_vertices);
void              render_data_free(seni_render_data* render_data);

void render_data_free_render_packets(seni_render_data* render_data);

seni_render_packet* add_render_packet(seni_render_data* render_data);
seni_render_packet* get_render_packet(seni_render_data* render_data, i32 index);
