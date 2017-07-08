#ifndef SENI_CONFIG_H
#define SENI_CONFIG_H

#define SENI_DEBUG_MODE

// define to run the test code, otherwise it will be printed
// (only used by test.c)
//
#define EXECUTE_BYTECODE

// define to perform run-time type checks on parameters passed into bindings
//
#define CHECK_STACK_ARGS

// define to print out opcodes while the bytecode is being executed
//
// #define TRACE_PRINT_OPCODES

#define MAX_WORD_LOOKUPS 128
#define MAX_KEYWORD_LOOKUPS 128
#define MAX_NATIVE_LOOKUPS 128

#define WORD_START 0
#define KEYWORD_START (WORD_START + MAX_WORD_LOOKUPS)
#define NATIVE_START (KEYWORD_START + MAX_KEYWORD_LOOKUPS)

// todo: errors probably shouldn't be silent when debug mode is switched off
//
#ifdef SENI_DEBUG_MODE
#include "seni_printf.h"
#define SENI_PRINT(f_, ...) seni_printf((f_), ##__VA_ARGS__);
#define SENI_LOG(f_, ...) seni_printf_log(__FILE__, __LINE__, (f_), ##__VA_ARGS__);
#define SENI_ERROR(f_, ...) seni_printf_error(__FILE__, __LINE__, (f_), ##__VA_ARGS__);
#else
#define SENI_PRINT(f_, ...)
#define SENI_LOG(f_, ...)
#define SENI_ERROR(f_, ...)
#endif
#endif // SENI_CONFIG_H
