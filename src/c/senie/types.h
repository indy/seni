#pragma once

#include <stdbool.h>
#include <stdint.h>

#ifndef NULL
#define NULL 0
#endif

typedef int8_t  i8;
typedef int16_t i16;
typedef int32_t i32;
typedef int64_t i64;

typedef uint8_t  u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

typedef float  f32;
typedef double f64;

typedef struct senie_bytecode        senie_bytecode;
typedef struct senie_colour          senie_colour;
typedef struct senie_colour_fn_state senie_colour_fn_state;
typedef struct senie_cursor          senie_cursor;
typedef struct senie_env             senie_env;
typedef struct senie_fn_info         senie_fn_info;
typedef struct senie_gene            senie_gene;
typedef struct senie_genotype        senie_genotype;
typedef struct senie_genotype_list   senie_genotype_list;
typedef struct senie_matrix          senie_matrix;
typedef struct senie_matrix_stack    senie_matrix_stack;
typedef struct senie_multistring     senie_multistring;
typedef struct senie_node            senie_node;
typedef struct senie_prng_state      senie_prng_state;
typedef struct senie_program         senie_program;
typedef struct senie_render_data     senie_render_data;
typedef struct senie_render_packet   senie_render_packet;
typedef struct senie_string_ref      senie_string_ref;
typedef struct senie_trait           senie_trait;
typedef struct senie_trait_list      senie_trait_list;
typedef struct senie_uv_mapping      senie_uv_mapping;
typedef struct senie_var             senie_var;
typedef struct senie_vm              senie_vm;
typedef struct senie_word_lut        senie_word_lut;
