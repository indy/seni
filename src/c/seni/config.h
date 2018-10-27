#pragma once

#define SEN_DEBUG_MODE

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

#define MAX_PREAMBLE_PROGRAM_SIZE 120
#define MAX_PROGRAM_SIZE 2048
#define MAX_TRAIT_PROGRAM_SIZE 256

#define VERTEX_PACKET_NUM_VERTICES 10000

// todo: errors probably shouldn't be silent when debug mode is switched off
//
#ifdef SEN_DEBUG_MODE
#include "printf.h"
#define SEN_PRINT(f_, ...) sen_printf((f_), ##__VA_ARGS__);
#define SEN_LOG(f_, ...) sen_printf_log(__FILE__, __LINE__, (f_), ##__VA_ARGS__);
#define SEN_ERROR(f_, ...) sen_printf_error(__FILE__, __LINE__, (f_), ##__VA_ARGS__);
#else
#define SEN_PRINT(f_, ...)
#define SEN_LOG(f_, ...)
#define SEN_ERROR(f_, ...)
#endif

#define RETURN_IF_NULL(expr, msg) \
  if ((expr) == NULL) {           \
    SEN_ERROR(msg);               \
    return NULL;                  \
  }
