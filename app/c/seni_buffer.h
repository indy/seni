#ifndef SENI_BUFFER_H
#define SENI_BUFFER_H

#include "seni_types.h"

// todo: very generic name, maybe rename to geometry buffer?
typedef struct seni_buffer {
  // max number of vertices that can fit in the buffer
  int max_vertices;

  // number of vertices actually in the buffer
  int num_vertices;

  f32* vbuf; // max_vertices * 2
  f32* cbuf; // max_vertices * 4
  f32* tbuf; // max_vertices * 2

} seni_buffer;

#endif
