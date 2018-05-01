#include "printf.h"

#include <string.h>
#ifdef SEN_BUILD_WASM
#include <webassembly.h>
#else
#include <stdio.h>
#endif

#define STB_SPRINTF_IMPLEMENTATION
#include "../lib/stb_sprintf.h"

#define SEN_PRINTF_BUFFER_SIZE 500

void sen_printf(char const* fmt, ...) {
  char buffer[SEN_PRINTF_BUFFER_SIZE];

  va_list va;
  va_start(va, fmt);
  stbsp_vsnprintf(buffer, SEN_PRINTF_BUFFER_SIZE, fmt, va);
  va_end(va);

#ifdef SEN_BUILD_WASM
  console_log("%s", buffer);
#else
  printf("%s\n", buffer);
#endif
}

int sen_sprintf(char* buf, int buffer_size, char const* fmt, ...) {
  va_list va;
  va_start(va, fmt);
  int len = stbsp_vsnprintf(buf, buffer_size, fmt, va);
  va_end(va);

  return len;
}

int sen_vsprintf(char* buf, int buffer_size, char const* fmt, va_list va) {
  int len = stbsp_vsnprintf(buf, buffer_size, fmt, va);

  return len;
}

void sen_fileline_sprintf(char*       buf,
                          int         buffer_size,
                          char*       file,
                          int         line,
                          char const* fmt,
                          va_list     va) {
  int   len = stbsp_snprintf(buf, buffer_size, "%s:%d: ", file, line);
  char* pp  = &(buf[len]);

  buffer_size -= len;
  stbsp_vsnprintf(pp, buffer_size, fmt, va);
}

void sen_printf_log(char* file, int line, char const* fmt, ...) {
  char buffer[SEN_PRINTF_BUFFER_SIZE];

  va_list va;
  va_start(va, fmt);
  sen_fileline_sprintf(buffer, SEN_PRINTF_BUFFER_SIZE, file, line, fmt, va);
  va_end(va);

#ifdef SEN_BUILD_WASM
  console_log("%s", buffer);
#else
  printf("%s\n", buffer);
#endif
}

void sen_printf_error(char* file, int line, char const* fmt, ...) {
  char buffer[SEN_PRINTF_BUFFER_SIZE];

  va_list va;
  va_start(va, fmt);
  sen_fileline_sprintf(buffer, SEN_PRINTF_BUFFER_SIZE, file, line, fmt, va);
  va_end(va);

#ifdef SEN_BUILD_WASM
  console_log("ERROR: %s", buffer);
#else
  printf("ERROR: %s\n", buffer);
#endif
}
