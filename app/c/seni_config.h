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


#ifdef SENI_DEBUG_MODE
#include <stdio.h>
#define SENI_ERROR(f_, ...) printf("ERROR: [%s %d] ", __FILE__, __LINE__); printf((f_), ##__VA_ARGS__); printf("\n")
#else
#define SENI_ERROR(f_, ...)
#endif


#endif // SENI_CONFIG_H
