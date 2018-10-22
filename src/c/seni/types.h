#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "error.h"
#include "result_macros.h"

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

typedef struct sen_bytecode        sen_bytecode;
typedef struct sen_colour          sen_colour;
typedef struct sen_colour_fn_state sen_colour_fn_state;
typedef struct sen_compiler_config sen_compiler_config;
typedef struct sen_cursor          sen_cursor;
typedef struct sen_env             sen_env;
typedef struct sen_fn_info         sen_fn_info;
typedef struct sen_gene            sen_gene;
typedef struct sen_genotype        sen_genotype;
typedef struct sen_genotype_list   sen_genotype_list;
typedef struct sen_matrix          sen_matrix;
typedef struct sen_matrix_stack    sen_matrix_stack;
typedef struct sen_multistring     sen_multistring;
typedef struct sen_node            sen_node;
typedef struct sen_prng_state      sen_prng_state;
typedef struct sen_program         sen_program;
typedef struct sen_render_data     sen_render_data;
typedef struct sen_render_packet   sen_render_packet;
typedef struct sen_string_ref      sen_string_ref;
typedef struct sen_trait           sen_trait;
typedef struct sen_trait_list      sen_trait_list;
typedef struct sen_uv_mapping      sen_uv_mapping;
typedef struct sen_var             sen_var;
typedef struct sen_vm              sen_vm;
typedef struct sen_word_lut        sen_word_lut;
