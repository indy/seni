#pragma once

#ifdef SEN_BUILD_WASM
#include <webassembly.h>
#else
#include <stdarg.h>
#include <stdio.h>
#endif

void sen_printf(char const* fmt, ...);
int  sen_sprintf(char* buf, int buffer_size, char const* fmt, ...);
int  sen_vsprintf(char* buf, int buffer_size, char const* fmt, va_list va);

void sen_printf_log(char* file, int line, char const* fmt, ...);
void sen_printf_error(char* file, int line, char const* fmt, ...);
