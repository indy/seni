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


// tests that pass on both windows and unix fail on macs
// they complain of ref_count becoming negative
// the fix is to add a useless printf("") to vector_ref_count_increment
// fuck knows why this is happening
// #define WHAT_THE_FUCK_MAC_HACK


#define MAX_WORD_LOOKUPS 128
#define MAX_KEYWORD_LOOKUPS 128
#define MAX_NATIVE_LOOKUPS 128

#define WORD_START 0
#define KEYWORD_START (WORD_START + MAX_WORD_LOOKUPS)
#define NATIVE_START (KEYWORD_START + MAX_KEYWORD_LOOKUPS)


#ifdef SENI_GEN_WASM
#include <webassembly.h>
#define SENI_PRINT(f_, ...) console_log((f_), ##__VA_ARGS__)
#else
#include <stdio.h>
#define SENI_PRINT(f_, ...) printf((f_), ##__VA_ARGS__)
#endif


// todo: errors probably shouldn't be silent when debug mode is switched off
#ifdef SENI_DEBUG_MODE

#ifdef SENI_GEN_WASM
#define SENI_ERROR(f_, ...) console_log("ERROR: [%s %d] ", __FILE__, __LINE__); console_log((f_), ##__VA_ARGS__); console_log("\n")
#else
#define SENI_ERROR(f_, ...) printf("ERROR: [%s %d] ", __FILE__, __LINE__); printf((f_), ##__VA_ARGS__); printf("\n")
#endif

#else
#define SENI_ERROR(f_, ...)
#endif


#endif // SENI_CONFIG_H
