#pragma once

#ifdef SENIE_BUILD_WASM
#include <webassembly.h>
#else
#include <stdarg.h>
#include <stdio.h>
#endif

void senie_printf(char const* fmt, ...);
int  senie_sprintf(char* buf, int buffer_size, char const* fmt, ...);
int  senie_vsprintf(char* buf, int buffer_size, char const* fmt, va_list va);

void senie_printf_log(char* file, int line, char const* fmt, ...);
void senie_printf_error(char* file, int line, char const* fmt, ...);
