#ifndef SENI_WASM_H
#define SENI_WASM_H

#include "seni_types.h"

typedef struct wasm_buffer {
  // the max number of vertices that can fit in the buffer
  int max_vertices;

  f32* vbuf; // max_vertices * 2
  f32* cbuf; // max_vertices * 4
  f32* tbuf; // max_vertices * 2

} wasm_buffer;

#endif
