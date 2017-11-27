#pragma once

#ifdef SENI_BUILD_WASM
#include <webassembly.h>
#else
#include <stdarg.h>
#include <stdio.h>
#endif

void seni_printf(char const *fmt, ...);
int  seni_sprintf(char *buf, int buffer_size, char const *fmt, ...);
int  seni_vsprintf(char *buf, int buffer_size, char const *fmt, va_list va);

void seni_printf_log(char *file, int line, char const *fmt, ...);
void seni_printf_error(char *file, int line, char const *fmt, ...);
