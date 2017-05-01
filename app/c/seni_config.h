#ifndef SENI_CONFIG_H
#define SENI_CONFIG_H

#define SENI_DEBUG_MODE

#ifdef SENI_DEBUG_MODE
#include <stdio.h>
#define SENI_ERROR(f_, ...) printf("ERROR: [%s %d] ", __FILE__, __LINE__); printf((f_), ##__VA_ARGS__); printf("\n")
#else
#define SENI_ERROR(f_, ...)
#endif

#endif // SENI_CONFIG_H
