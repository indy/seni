#pragma once

#define SENIE_DEBUG_MODE

// define to perform run-time type checks on parameters passed into bindings
//
#define CHECK_STACK_ARGS

// define to print out opcodes while the bytecode is being executed
//
// #define TRACE_PRINT_OPCODES

#define MAX_WORD_LOOKUPS 128
#define MAX_KEYWORD_LOOKUPS 192
#define MAX_NATIVE_LOOKUPS 128

#define WORD_START 0
#define KEYWORD_START (WORD_START + MAX_WORD_LOOKUPS)
#define NATIVE_START (KEYWORD_START + MAX_KEYWORD_LOOKUPS)

#define MAX_PROGRAM_SIZE 2048
#define MAX_TRAIT_PROGRAM_SIZE 256

#define VERTEX_PACKET_NUM_VERTICES 10000

// todo: errors probably shouldn't be silent when debug mode is switched off
//
#ifdef SENIE_DEBUG_MODE
#include "printf.h"
#define SENIE_PRINT(f_, ...) senie_printf((f_), ##__VA_ARGS__);
#define SENIE_LOG(f_, ...) senie_printf_log(__FILE__, __LINE__, (f_), ##__VA_ARGS__);
#define SENIE_ERROR(f_, ...) senie_printf_error(__FILE__, __LINE__, (f_), ##__VA_ARGS__);
#else
#define SENIE_PRINT(f_, ...)
#define SENIE_LOG(f_, ...)
#define SENIE_ERROR(f_, ...)
#endif

#define RETURN_IF_NULL(expr, msg) \
  if ((expr) == NULL) {           \
    SENIE_ERROR(msg);             \
    return NULL;                  \
  }
