#pragma once

void seni_printf(char const * fmt, ... );
int seni_sprintf(char *buf, int buffer_size, char const * fmt, ... );

void seni_printf_log(char *file, int line, char const * fmt, ... );
void seni_printf_error(char *file, int line, char const * fmt, ... );

