#pragma once

#include <stdint.h>
#include <stdbool.h>

#ifndef NULL
#define NULL 0
#endif

typedef int8_t   i8;
typedef int16_t  i16;
typedef int32_t  i32;
typedef int64_t  i64;

typedef uint8_t  u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

typedef float    f32;
typedef double   f64;

typedef struct seni_bytecode seni_bytecode;
typedef struct seni_colour seni_colour;
typedef struct seni_colour_fn_state seni_colour_fn_state;
typedef struct seni_env seni_env;
typedef struct seni_fn_info seni_fn_info;
typedef struct seni_gene seni_gene;
typedef struct seni_genotype seni_genotype;
typedef struct seni_genotype_list seni_genotype_list;
typedef struct seni_matrix seni_matrix;
typedef struct seni_matrix_stack seni_matrix_stack;
typedef struct seni_multistring_buffer seni_multistring_buffer;
typedef struct seni_node seni_node;
typedef struct seni_prng_state seni_prng_state;
typedef struct seni_program seni_program;
typedef struct seni_render_data seni_render_data;
typedef struct seni_render_packet seni_render_packet;
typedef struct seni_string_ref seni_string_ref;
typedef struct seni_text_buffer seni_text_buffer;
typedef struct seni_trait seni_trait;
typedef struct seni_trait_list seni_trait_list;
typedef struct seni_uv_mapping seni_uv_mapping;
typedef struct seni_var seni_var;
typedef struct seni_vm seni_vm;
typedef struct seni_word_lut seni_word_lut;
