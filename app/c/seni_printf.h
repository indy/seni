#ifndef SENI_PRINTF_H
#define SENI_PRINTF_H

void seni_printf(char const * fmt, ... );
int seni_sprintf(char *buf, int buffer_size, char const * fmt, ... );

void seni_printf_log(char *file, int line, char const * fmt, ... );
void seni_printf_error(char *file, int line, char const * fmt, ... );

#endif
