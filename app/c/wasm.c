#include <emscripten/emscripten.h>
#include "seni.h"

f32 mult = 3.2f;
int lensub = 0;

EMSCRIPTEN_KEEPALIVE
int buffer_fill(f32* array, int length, char *script)
{
  printf("the script is %s\n", script);
  
  printf("array length is %d\n", length);

  for (int i=0; i<length; i++) {
    printf("array[%d] = %.2f\n", i, array[i]);
    array[i] = (f32)i * mult;
  }

  int retlength = length - lensub;
  lensub++;
  mult += 1.2f;
  
  return retlength;
}

// returns the number of vertices to render
EMSCRIPTEN_KEEPALIVE
int render(f32* vbuf, f32* cbuf, f32* tbuf, int length, char *script)
{
  printf("the script is %s\n", script);
  //printf("array length is %d\n", length);


  // return 3 vectors
  
  int i;
  int num_vertices = 3;


  // v0
  vbuf[0] = 500.0f;
  vbuf[1] = 500.0f;

  // v1
  vbuf[2] = 300.0f;
  vbuf[3] = 300.0f;

  // v2
  vbuf[4] = 800.0f;
  vbuf[5] = 100.0f;

  // --------------------------------------------------

  // v0
  cbuf[0] = 0.0f;
  cbuf[1] = 0.0f;
  cbuf[2] = 0.0f;
  cbuf[3] = 1.0f;
  
  // v1
  cbuf[4] = 0.0f;
  cbuf[5] = 0.0f;
  cbuf[6] = 0.0f;
  cbuf[7] = 1.0f;
  
  // v2
  cbuf[8]  = 0.0f;
  cbuf[9]  = 0.0f;
  cbuf[10] = 0.0f;
  cbuf[11] = 1.0f;

  // ------------------------------------------------

  // v0
  f32 a = 1.0f / 1024.0f;
  f32 b = 1.0f / 1024.0f;
  
  tbuf[0] = a;
  tbuf[1] = a;
  
  // v1
  tbuf[2] = b;
  tbuf[3] = a;
  
  // v2
  tbuf[4] = a;
  tbuf[5] = b;
  
  return num_vertices;
}
