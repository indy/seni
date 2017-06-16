/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"

#include "seni_config.h"
#include "seni_types.h"
#include "seni_mathutil.h"
#include "seni_lang.h"
#include "seni_bind.h"
#include "seni_uv_mapper.h"
#include "seni_colour.h"
#include "seni_prng.h"

#include "time.h"
#include "stdio.h"
#include <stdlib.h>
#include <string.h>

/* way of working with boolean and TEST macros */
bool test_true = true;
bool test_false = false;

/* required by unity */
void setUp(void) { }
void tearDown(void) { }

void test_mathutil(void)
{
  TEST_ASSERT_EQUAL_FLOAT(1.5f, deg_to_rad(rad_to_deg(1.5f)));
  TEST_ASSERT_EQUAL_FLOAT(0.44444f, mc_m(1.0f, 1.0f, 10.0f, 5.0f));
  TEST_ASSERT_EQUAL_FLOAT(0.55556f, mc_c(1.0f, 1.0f, 0.444444f));
}

seni_node *assert_parser_node_raw(seni_node *node, seni_node_type type)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));
  return node->next;
}

seni_node *assert_parser_node_i32(seni_node *node, seni_node_type type, i32 val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));
  TEST_ASSERT_EQUAL(val, node->value.i);
  return node->next;
}

seni_node *assert_parser_node_f32(seni_node *node, f32 val)
{
  TEST_ASSERT_EQUAL_MESSAGE(NODE_FLOAT, node->type, node_type_name(node));
  TEST_ASSERT_EQUAL_FLOAT(val, node->value.f);
  return node->next;
}

seni_node *assert_parser_node_str(seni_node *node, seni_node_type type, char *val)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));
  TEST_ASSERT_EQUAL_STRING(val, node->value.s);
  return node->next;
}

seni_node *assert_parser_node_txt(seni_node *node, seni_node_type type, char *val, seni_word_lut *wlut)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));

  char *c = wlut->word[node->value.i];
  TEST_ASSERT_EQUAL_STRING(val, c);
  
  return node->next;
}

#define PARSE(EXPR) wl = wlut_allocate(); \
  nodes = parser_parse(wl, EXPR)

#define PARSE_CLEANUP wlut_free(wl); \
  parser_free_nodes(nodes)


void test_parser(void)
{
  seni_node *nodes, *iter, *iter2;
  seni_word_lut *wl;

  PARSE("hello");
  assert_parser_node_txt(nodes, NODE_NAME, "hello", wl);
  PARSE_CLEANUP;

  PARSE("5");
  assert_parser_node_f32(nodes, 5);
  PARSE_CLEANUP;

  PARSE("(4)");
  assert_parser_node_raw(nodes, NODE_LIST);
  PARSE_CLEANUP;

  PARSE("true");
  assert_parser_node_i32(nodes, NODE_BOOLEAN, true);
  PARSE_CLEANUP;

  PARSE("false");
  assert_parser_node_i32(nodes, NODE_BOOLEAN, false);
  PARSE_CLEANUP;

  PARSE("(add 1 2)");
  iter = nodes->value.first_child;
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "add", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 1);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 2);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;

  PARSE("[add 9 8 (foo)]");
  assert_parser_node_raw(nodes, NODE_VECTOR);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "add", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 9);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 8);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_raw(iter, NODE_LIST);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;

  PARSE(";[add 9 8 (foo)]");
  assert_parser_node_str(nodes, NODE_COMMENT, ";[add 9 8 (foo)]");
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;

  PARSE("'(runall \"shabba\") ; woohoo");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "quote", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter;
  iter = assert_parser_node_raw(iter, NODE_LIST);
  TEST_ASSERT_NULL(iter);
  iter = iter2->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "runall", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_STRING, "shabba", wl);
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_str(iter, NODE_COMMENT, "; woohoo");
  TEST_ASSERT_NULL(iter);
  PARSE_CLEANUP;

  PARSE("(fun i: 42 f: 12.34)");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "fun", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_LABEL, "i", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 42);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_LABEL, "f", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 12.34f);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;

  PARSE("(a 1) (b 2)");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "a", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 1);
  TEST_ASSERT_NULL(iter);
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  assert_parser_node_raw(iter, NODE_LIST);
  iter = iter->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "b", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 2);
  TEST_ASSERT_NULL(iter);
  PARSE_CLEANUP;

  PARSE("(a {[1 2]})");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "a", wl);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter; // the vector
  iter = assert_parser_node_raw(iter, NODE_VECTOR);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_EQUAL(test_true, iter2->alterable);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;
}

void assert_seni_var_f32(seni_var *var, seni_var_type type, f32 f)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var));
  TEST_ASSERT_EQUAL_FLOAT(f, var->value.f);
}

void assert_seni_var_v2(seni_var *var, f32 a, f32 b)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VEC_HEAD, var->type, "VAR_VEC_HEAD");
  seni_var *rc = var->value.v;
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VEC_RC, rc->type, "VAR_VEC_RC");

  seni_var *val = rc->next;
  TEST_ASSERT_EQUAL_FLOAT(a, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(b, val->value.f);
}

void assert_seni_var_v4(seni_var *var, f32 a, f32 b, f32 c, f32 d)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VEC_HEAD, var->type, "VAR_VEC_HEAD");

  seni_var *rc = var->value.v;
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VEC_RC, rc->type, "VAR_VEC_RC");

  seni_var *val = rc->next;
  TEST_ASSERT_EQUAL_FLOAT(a, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(b, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(c, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(d, val->value.f);
}

void assert_seni_var_col(seni_var *var, i32 format, f32 a, f32 b, f32 c, f32 d)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_COLOUR, var->type, "VAR_COLOUR");

  seni_colour *colour = var->value.c;

  TEST_ASSERT_EQUAL(format, (i32)colour->format);

  TEST_ASSERT_FLOAT_WITHIN(0.1f, a, colour->element[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, b, colour->element[1]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, c, colour->element[2]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, d, colour->element[3]);
}

void assert_seni_var_f32_within(seni_var *var, seni_var_type type, f32 f, f32 tolerance)
{
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var));
  TEST_ASSERT_FLOAT_WITHIN(tolerance, f, var->value.f);
}

void assert_seni_var_bool(seni_var *var, bool b)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_BOOLEAN, var->type, var_type_name(var));
  TEST_ASSERT_EQUAL(b ? 1 : 0, var->value.i);
}

void test_uv_mapper(void)
{
  init_uv_mapper();

  seni_uv_mapping *flat = get_uv_mapping(BRUSH_FLAT, 0, true);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.0f, flat->width_scale);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 2.0f / 1024.0f, flat->map[0].x);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.0f / 1024.0f, flat->map[0].y);
  
  TEST_ASSERT_NULL(get_uv_mapping(BRUSH_FLAT, 1, false)); // out of range

  seni_uv_mapping *c = get_uv_mapping(BRUSH_C, 8, false);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.1f, c->width_scale);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 326.0f / 1024.0f, c->map[0].x);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 556.0f / 1024.0f, c->map[0].y);

  free_uv_mapper();
}

void assert_colour(seni_colour *expected, seni_colour *colour)
{
  TEST_ASSERT_EQUAL(expected->format, colour->format);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, expected->element[0], colour->element[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, expected->element[1], colour->element[1]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, expected->element[2], colour->element[2]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, expected->element[3], colour->element[3]);
}

void test_colour(void)
{
  {
    seni_colour *c = colour_construct(RGB, 0.0f, 0.0f, 0.0f, 1.0f);
    TEST_ASSERT_EQUAL(RGB, c->format);
    TEST_ASSERT_EQUAL_FLOAT(0.0f, c->element[0]);
    TEST_ASSERT_EQUAL_FLOAT(0.0f, c->element[1]);
    TEST_ASSERT_EQUAL_FLOAT(0.0f, c->element[2]);
    TEST_ASSERT_EQUAL_FLOAT(1.0f, c->element[3]);
    colour_free(c);
  }

  {
    seni_colour *rgb = colour_construct(RGB, 0.2f, 0.1f, 0.5f, 1.0f);
    seni_colour *hsl = colour_construct(HSL, 255.0f, 0.6666f, 0.3f, 1.0f);
    seni_colour *lab = colour_construct(LAB, 19.9072f, 39.6375f, -52.7720f, 1.0f);

    seni_colour res;

    assert_colour(rgb, colour_clone_as(&res, rgb, RGB));
    assert_colour(hsl, colour_clone_as(&res, rgb, HSL));
    assert_colour(lab, colour_clone_as(&res, rgb, LAB));

    assert_colour(rgb, colour_clone_as(&res, hsl, RGB));
    assert_colour(hsl, colour_clone_as(&res, hsl, HSL));
    assert_colour(lab, colour_clone_as(&res, hsl, LAB));

    assert_colour(rgb, colour_clone_as(&res, lab, RGB));
    assert_colour(hsl, colour_clone_as(&res, lab, HSL));
    assert_colour(lab, colour_clone_as(&res, lab, LAB));

    colour_free(rgb);
    colour_free(hsl);
    colour_free(lab);
  }
}

// void minmax(f32 lacunarity, f32 gain, f32 offset, i32 octaves)
// {
//   f32 w, min, max;

//   i32 i, j;
//   min = 10000.0f;
//   max = -10000.0f;
//   for (j = 0; j < 500; j++) {
//     for (i = 0; i < 500; i++) {
//       w = seni_perlin((f32)j / 4701.0f, (f32)i / 41.0f, ((f32)j / 471.0f) + ((f32)i / 471.0f));
//       max = w > max ? w : max;
//       min = w < min ? w : min;
//     }
//   }
//   printf("min %f, max %f\n", min, max);
// }

void test_prng(void)
{
  // seni_prng_state prng_state;
  // seni_prng_set_state(&prng_state, 65493);

  // f32 a = seni_prng_f32(&prng_state);
  // f32 b = seni_prng_f32(&prng_state);
  // printf("a = %.3f b = %.3f\n", a, b);


  

  // u32 u, umin, umax;
  // umin = 10000;
  // umax = 0;
  // for (i32 i = 0; i < 100; i++) {
  //   u = seni_prng_u32(&prng_state1, 10);
  //   umax = u > umax ? u : umax;
  //   umin = u < umin ? u : umin;
  // }
  // printf("u32 min %d, max %d\n", umin, umax);

  // f32 w, min, max;
  // min = 10000.0f;
  // max = -10000.0f;
  // for (i32 i = 0; i < 10000; i++) {
  //   w = seni_prng_f32(&prng_state1);
  //   max = w > max ? w : max;
  //   min = w < min ? w : min;
  // }

  // printf("min %f, max %f\n", min, max);

  // TEST_ASSERT_EQUAL(230, u);

  // minmax(1.0f, 1.0f, 0.0f, 1);
  // minmax(1.0f, 1.0f, 1.0f, 1);
  // minmax(1.0f, 1.0f, 2.0f, 1);
  // minmax(1.0f, 1.0f, 3.0f, 1);

  // printf("%f\n", seni_perlin(0.0001f, 0.1f, 0.5f));
  // printf("%f\n", seni_perlin(90.4f, 13.1f, 4394.0f));
  // printf("%f\n", seni_perlin(0.122f, 0.1f, 0.3f));
  // printf("%f\n", seni_perlin(0.1f, 0.1f, 0.1f));
  
}

// --------------------------------------------------

seni_word_lut *setup_vm_wl(seni_env *e)
{
  seni_word_lut *wl = wlut_allocate();
  // add keywords to the seni_word_lut and setup function pointers within the interpreter
  declare_bindings(wl, e);

  return wl;
}

void shutdown_interpreter_test(seni_word_lut *wl, seni_node *ast)
{
  wlut_free(wl);
  parser_free_nodes(ast);
}

// debug version of VM_COMPILE - prints the bytecode
//
#define DVM_COMPILE(EXPR) seni_word_lut *wl = NULL;       \
  seni_env *e = NULL;                                     \
  seni_node *ast = NULL;                                  \
  seni_program *prog = NULL;                              \
  seni_vm *vm = NULL;                                     \
  e = env_construct();                                    \
  wl = setup_vm_wl(e);                                    \
  ast = parser_parse(wl, EXPR);                           \
  prog = program_allocate(256);                           \
  prog->wl = wl;                                          \
  prog->env = e;                                          \
  compiler_compile(ast, prog);                            \
  vm = vm_construct(STACK_SIZE,MEMORY_SIZE);              \
  printf("%s\n", EXPR);                                   \
  pretty_print_program(prog);

// --------------------------------------------------

// eval version of VM_COMPILE - evals and compares result to an int
//
#define EVM_COMPILE(EXPR) seni_word_lut *wl = NULL;         \
  seni_env *e = NULL;                                       \
  seni_node *ast = NULL;                                    \
  seni_program *prog = NULL;                                \
  seni_vm *vm = NULL;                                       \
  e = env_construct();                                      \
  wl = setup_vm_wl(e);                                      \
  ast = parser_parse(wl, EXPR);                             \
  prog = program_allocate(256);                             \
  prog->wl = wl;                                            \
  prog->env = e;                                            \
  compiler_compile(ast, prog);                              \
  vm = vm_construct(STACK_SIZE,MEMORY_SIZE);                \
  vm_interpret(vm, prog)


#ifdef SENI_DEBUG_MODE
#define VM_HEAP_SLAB_CHECK TEST_ASSERT_EQUAL_MESSAGE(0, vm->heap_slab_info.delta, "vm heap slab leak")
#else
#define VM_HEAP_SLAB_CHECK
#endif

#ifdef SENI_DEBUG_MODE
#define VM_COLOUR_SLAB_CHECK TEST_ASSERT_EQUAL_MESSAGE(0, vm->colour_slab_info.delta, "vm colour slab leak")
#else
#define VM_COLOUR_SLAB_CHECK
#endif

#define VM_TEST_FLOAT(RES) assert_seni_var_f32(stack_peek(vm), VAR_FLOAT, RES)
#define VM_TEST_BOOL(RES) assert_seni_var_bool(stack_peek(vm), RES)
#define VM_TEST_VEC2(A,B) assert_seni_var_v2(stack_peek(vm), A, B)
#define VM_TEST_VEC4(A,B,C,D) assert_seni_var_v4(stack_peek(vm), A, B, C, D)
#define VM_TEST_COL(F,A,B,C,D) assert_seni_var_col(stack_peek(vm), F, A, B, C, D)


#define VM_CLEANUP shutdown_interpreter_test(wl, ast);  \
  program_free(prog);                                   \
  env_free(e);                                          \
  vm_free(vm)


// COMPILE macros that eval and compare results
//
// 0 == print out bytecode, 1 == execute bytecode
#ifdef EXECUTE_BYTECODE

// ************************************************
// TODO: use the above definition of VM_COMPILE_INT
// ************************************************
#define VM_COMPILE_F32(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_FLOAT(RES);VM_HEAP_SLAB_CHECK;VM_CLEANUP;}
#define VM_COMPILE_BOOL(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_BOOL(RES);VM_HEAP_SLAB_CHECK;VM_CLEANUP;}
#define VM_COMPILE_VEC2(EXPR,A,B) {EVM_COMPILE(EXPR);VM_TEST_VEC2(A,B);VM_CLEANUP;}
#define VM_COMPILE_VEC4(EXPR,A,B,C,D) {EVM_COMPILE(EXPR);VM_TEST_VEC4(A,B,C,D);VM_CLEANUP;}
#define VM_COMPILE_COL(EXPR,F,A,B,C,D) {EVM_COMPILE(EXPR);VM_TEST_COL(F,A,B,C,D);VM_COLOUR_SLAB_CHECK;VM_CLEANUP;}
// don't perform a heap check as we're assuming that the expr will be leaky
#define VM_COMPILE_F32_L(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_FLOAT(RES);VM_CLEANUP;}
#define VM_COMPILE_COL_L(EXPR,F,A,B,C,D) {EVM_COMPILE(EXPR);VM_TEST_COL(F,A,B,C,D);VM_CLEANUP;}

#else
// COMPILE macros that print out bytecode
//
#define VM_COMPILE_F32(EXPR,_) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_BOOL(EXPR,_) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_VEC2(EXPR,_,_1) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_VEC4(EXPR,_,_1,_2,_3) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_COL(EXPR,_,_1,_2,_3,_4) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_F32_L(EXPR,_) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_COL_L(EXPR,F,A,B,C,D) {EVM_COMPILE(EXPR);VM_CLEANUP;}
#endif
// --------------------------------------------------

void timing(void)
{
  clock_t start, diff;
  int msec;
  {
    start = clock();
    //VM_COMPILE_F32("(loop (x from: 0 to: 1000000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1)) 4", 4);

    VM_COMPILE_F32("(loop (x from: 0 to: 10000) (loop (y from: 0 to: 1000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (+ 3 4))) 9", 9);
    diff = clock() - start;
    msec = diff * 1000 / CLOCKS_PER_SEC;
    printf("VM Time taken %d seconds %d milliseconds\n", msec/1000, msec%1000);
  }
}

void test_vm_bytecode(void)
{
  VM_COMPILE_F32("(define a 42) (define b 52) 10", 10);
  VM_COMPILE_F32("(define a 6) (define b 7) (+ a b)", 13);
  VM_COMPILE_F32("(define a 8 b 9) (+ a b)", 17);
  VM_COMPILE_F32("(+ 3 4)", 7);
  VM_COMPILE_F32("(+ 3 4 5)", 12);
  VM_COMPILE_F32("(+ 3 4 5 6)", 18);
  VM_COMPILE_F32("(- (+ 1 2) 3)", 0);
  VM_COMPILE_F32("(* 3 4)", 12);
  VM_COMPILE_F32("(* 3 4 5)", 60);
  VM_COMPILE_F32("(/ 90 10)", 9);
  VM_COMPILE_F32("(/ 90 10 3)", 3);
  VM_COMPILE_BOOL("(> 5 10)", false);
  VM_COMPILE_BOOL("(< 5 10)", true);
  VM_COMPILE_BOOL("(< 1 2 3 4 5 10)", true);
  VM_COMPILE_BOOL("(= 2 2)", true);
  VM_COMPILE_BOOL("(= 1 2)", false);
  VM_COMPILE_BOOL("(= 1 1 1 1 1 2)", false);
  VM_COMPILE_BOOL("(and (< 1 2) (< 3 4))", true);
  VM_COMPILE_BOOL("(and (< 1 2) (< 3 4) (< 5 6) (< 7 8))", true);
  VM_COMPILE_BOOL("(and (< 1 2) (> 3 4))", false);
  VM_COMPILE_BOOL("(or (< 1 2) (> 3 4))", true);
  VM_COMPILE_BOOL("(not (> 1 10))", true);
  VM_COMPILE_BOOL("(and (or (< 1 2) (> 3 4)) (not (> 1 10)))", true);

  VM_COMPILE_F32("(if (> 400 200) 66)", 66);
  VM_COMPILE_F32("(if (> 200 100) 12 24)", 12);
  VM_COMPILE_F32("(if (< 200 100) 12 24)", 24);
  VM_COMPILE_BOOL("(if (> 400 200) (= 50 50))", true);
  VM_COMPILE_BOOL("(if (> 99 88) (= 3 4) (= 5 5))", false);
  VM_COMPILE_BOOL("(if (< 99 88) (= 3 4) (= 5 5))", true);

  VM_COMPILE_F32("(loop (x from: 0 to: 5) (+ 42 38)) 9", 9);
  VM_COMPILE_F32("(loop (x from: 0 to: 5) (loop (y from: 0 to: 5) (+ 3 4))) 9", 9);
}

void test_vm_callret(void)
{
  // basic invocation
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: 3)", 8);
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: (+ 3 4))", 12); // calc required for value
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 xxx: 3)", 13); // non-existent argument
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder)", 17); // only default arguments
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 10)", 18); // missing argument
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder b: 20)", 29); // missing argument

  VM_COMPILE_F32("(fn (p2 a: 1) (+ a 2)) (fn (p3 a: 1) (+ a 3)) (+ (p2 a: 5) (p3 a: 10))", 20);
  VM_COMPILE_F32("(fn (p2 a: 1) (+ a 2)) (fn (p3 a: 1) (+ a 3)) (p2 a: (p3 a: 10))", 15);

  // functions calling functions
  VM_COMPILE_F32("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z)))      (x)",       6);
  VM_COMPILE_F32("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z a: 5))) (x)",      10);
  VM_COMPILE_F32("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z a: 5))) (x c: 5)", 12);
  
  // function calling another function, passing on one of it's local variables
  // (make use of the hop_back method of referring to the correct LOCAL frame)
  VM_COMPILE_F32("(fn (z a: 1) (+ a 5)) (fn (y) (define x 10) (z a: x)) (y)", 15);
  VM_COMPILE_F32("(fn (z a: 1) (+ a 5)) (fn (zz a: 1) (+ a 9))(fn (y) (define x 10) (z a: (zz a: x))) (y)", 24);

  // function referencing a global
  VM_COMPILE_F32("(define gs 30)(fn (foo at: 0) (+ at gs))(foo at: 10)", 40);

  // global references a function, function references a global
  VM_COMPILE_F32("(define a 5 b (acc n: 2)) (fn (acc n: 0) (+ n a)) (+ a b)", 12);
}

void test_vm_native(void)
{
  // call native functions that have vector arguments (tests ref counting)
  VM_COMPILE_F32("(nth n: 0 from: [22 33])", 22);
  VM_COMPILE_F32_L("(define aa [22 33]) (nth n: 0 from: aa)", 22);  
  VM_COMPILE_F32("(fn (x) (nth n: 0 from: [22 33])) (x)", 22);
  VM_COMPILE_F32("(math/distance vec1: [0 0] vec2: [0 20])", 20.0f);
}

void test_vm_destructure(void)
{
  VM_COMPILE_F32("(define [a b] [22 33]) (- b a)", 11);
  VM_COMPILE_F32("(define [a b c] [22 33 44]) (+ a b c)", 99);
}

void test_vm_vector(void)
{
  VM_COMPILE_VEC2("[4 5]", 4, 5);

  VM_COMPILE_F32("(loop (x from: 0 to: 5) [1 2]) 9", 9);

  // explicitly defined vector is returned
  VM_COMPILE_VEC2("(fn (f a: 3) [1 2]) (fn (x) (f)) (x)", 1, 2);

  // local var in function is returned
  VM_COMPILE_VEC2("(fn (f a: 3) (define b [1 2]) b) (fn (x) (f)) (x)", 1, 2);

  // local var in function is not returned
  VM_COMPILE_F32("(fn (f a: 3) (define b [1 2]) 55) (fn (x) (f)) (x)", 55);

  // default argument for function is returned
  VM_COMPILE_VEC2("(fn (f a: [1 2]) a) (fn (x) (f)) (x)", 1, 2);

  // default argument for function is not returned
  VM_COMPILE_F32("(fn (f a: [1 2]) 3) (fn (x) (f)) (x)", 3);

  // default argument for function is not returned and
  // it's called with an explicitly declared vector
  VM_COMPILE_F32("(fn (f a: [1 2]) 3) (fn (x) (f a: [3 4])) (x)", 3);

  // default argument for function is not returned and
  // it's called with an unused argument
  VM_COMPILE_F32("(fn (f a: [1 2]) 3) (fn (x) (f z: [3 4])) (x)", 3);

  // default argument for function is not returned
  VM_COMPILE_F32("(fn (f a: [1 2]) a) (fn (x) (f a: 5)) (x)", 5);
  
  // argument into function is returned
  VM_COMPILE_VEC2("(fn (f a: [3 4]) a) (fn (x) (f a: [1 2])) (x)", 1, 2);
}

void test_vm_col_rgb(void)
{
  VM_COMPILE_COL_L("(col/rgb r: 0.1 g: 0.2 b: 0.3 alpha 0.4)", RGB, 0.1f, 0.2f, 0.3f, 0.4f);
  // checking colour_avail
  VM_COMPILE_F32("(fn (f) (col/rgb r: 0.1 g: 0.2 b: 0.3 alpha 0.4) 5) (f)", 5.0f);
}

void test_vm_math(void)
{
  VM_COMPILE_F32("(math/distance vec1: [0 0] vec2: [0 20])", 20.0f);
  VM_COMPILE_F32("(math/distance vec1: [0 5] vec2: [0 20])", 15.0f);
  VM_COMPILE_F32("(math/clamp val: 0.1 min: 0.0 max: 5)", 0.1f);
  VM_COMPILE_F32("(math/clamp val: -0.1 min: 0.0 max: 5)", 0.0f);
  VM_COMPILE_F32("(math/clamp val: 10 min: 0.0 max: 5)", 5.0f);
}

void test_vm_prng(void)
{
  // leaky because the global rng is a vector
  //
  VM_COMPILE_F32_L("(define rng (prng/build seed: 43215 min: 5 max: 20)) (prng/take-1 from: rng)", 16.32348f);
  // state of rng is changing, returning a different number than previous tests
  VM_COMPILE_F32_L("(define rng (prng/build seed: 43215 min: 5 max: 20)) (prng/take-1 from: rng) (prng/take-1 from: rng)", 13.451335f);

  // wrapped in a function so that it's not leaky
  VM_COMPILE_F32("(fn (x) (define rng (prng/build seed: 43215 min: 5 max: 20)) (prng/take-1 from: rng)) (x)", 16.32348f);
  // state of rng is changing, returning a different number than previous tests
  VM_COMPILE_F32("(fn (x) (define rng (prng/build seed: 43215 min: 5 max: 20)) (prng/take-1 from: rng) (prng/take-1 from: rng)) (x)", 13.451335f);

  // prng/take returning a vector
  VM_COMPILE_F32_L("(define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/take num: 3 from: rng)) (nth n: 0 from: foo)", 0.27065f);
  VM_COMPILE_F32_L("(define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/take num: 3 from: rng)) (nth n: 1 from: foo)", 0.75259f);
  VM_COMPILE_F32_L("(define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/take num: 3 from: rng)) (nth n: 2 from: foo)", -0.141389f);

  // non-leaky version of above
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/take num: 3 from: rng)) (nth n: 0 from: foo)) (x)", 0.27065f);
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/take num: 3 from: rng)) (nth n: 1 from: foo)) (x)", 0.75259f);
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/take num: 3 from: rng)) (nth n: 2 from: foo)) (x)", -0.141389f);

  // prng, destructuring, multiple args to '+'
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define [a b c] (prng/take num: 3 from: rng)) (+ a b c)) (x)", 0.881854f);
  
}

// code that exposed bugs - but was later fixed
void test_vm_bugs(void)
{
  // messed up decrementing ref counts for colours
  VM_COMPILE_F32("(fn (x colour: (col/rgb)) (+ 1 1)) (x colour: (col/rgb))", 2.0f);

  // tmp in interpreter was decrementing ref count of previously held value
     VM_COMPILE_F32("(fn (huh at: 0) 4)\
 (fn (x colour: (col/rgb)              \
        volatility: 0                  \
        passes: 1                      \
        seed: 341)                     \
   42)                                 \
 (x colour: (col/rgb))                 \
 (huh at: 5)", 4.0f);

  VM_COMPILE_F32("(fn (f) (define rng (prng/build min: -1 max: 1 seed: 111)) (loop (i from: 0 to: 2) (define [rr rx ry] (prng/take num: 3 from: rng)))) (f) 1", 1.0f);
}

void test_temp(void)
{

}

int main(void)
{
  // timing();
    
  UNITY_BEGIN();

  //RUN_TEST(debug_lang_interpret_mem); // for debugging/development
  
  RUN_TEST(test_mathutil);
  RUN_TEST(test_parser);
  RUN_TEST(test_uv_mapper);
  RUN_TEST(test_colour);
  
  // vm
  RUN_TEST(test_vm_bytecode);
  RUN_TEST(test_vm_callret);
  RUN_TEST(test_vm_native);  
  RUN_TEST(test_vm_destructure);
  RUN_TEST(test_vm_vector);
  RUN_TEST(test_vm_col_rgb);
  RUN_TEST(test_vm_math);
  RUN_TEST(test_vm_prng);
  RUN_TEST(test_vm_bugs);

  //RUN_TEST(test_temp);
  
  return UNITY_END();
}
