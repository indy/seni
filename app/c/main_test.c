/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"

#include "seni_bind.h"
#include "seni_colour.h"
#include "seni_config.h"
#include "seni_keyword_iname.h"
#include "seni_lang.h"
#include "seni_mathutil.h"
#include "seni_parser.h"
#include "seni_prng.h"
#include "seni_shapes.h"
#include "seni_strtof.h"
#include "seni_timing.h"
#include "seni_types.h"
#include "seni_unparser.h"
#include "seni_uv_mapper.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"

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

  // note: can parse string but it isn't being used - should it be removed?
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

void assert_seni_var_v4(seni_var *var, f32 a, f32 b, f32 c, f32 d)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VECTOR, var->type, "VAR_VECTOR");

  seni_var *val = var->value.v;
  TEST_ASSERT_EQUAL_FLOAT(a, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(b, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(c, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(d, val->value.f);
}

void assert_seni_var_v5(seni_var *var, f32 a, f32 b, f32 c, f32 d, f32 e)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VECTOR, var->type, "VAR_VECTOR");

  seni_var *val = var->value.v;
  TEST_ASSERT_EQUAL_FLOAT(a, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(b, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(c, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(d, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(e, val->value.f);
}

void assert_seni_var_col(seni_var *var, i32 format, f32 a, f32 b, f32 c, f32 d)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_COLOUR, var->type, "VAR_COLOUR");

  // seni_colour *colour = var->value.c;

  TEST_ASSERT_EQUAL(format, (i32)var->value.i);

  TEST_ASSERT_FLOAT_WITHIN(0.1f, a, var->f32_array[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, b, var->f32_array[1]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, c, var->f32_array[2]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, d, var->f32_array[3]);
}

void assert_seni_var_2d(seni_var *var, f32 a, f32 b)
{
  TEST_ASSERT_EQUAL_MESSAGE(VAR_2D, var->type, "VAR_2D");

  TEST_ASSERT_FLOAT_WITHIN(0.1f, a, var->f32_array[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, b, var->f32_array[1]);
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
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 2.0f / 1024.0f, flat->map[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.0f / 1024.0f, flat->map[1]);
  
  TEST_ASSERT_NULL(get_uv_mapping(BRUSH_FLAT, 1, false)); // out of range

  seni_uv_mapping *c = get_uv_mapping(BRUSH_C, 8, false);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.1f, c->width_scale);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 326.0f / 1024.0f, c->map[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 556.0f / 1024.0f, c->map[1]);

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

void test_strtof(void)
{
  char **end = NULL;

  TEST_ASSERT_EQUAL_FLOAT(3.14f, seni_strtof("3.14", end));
  TEST_ASSERT_EQUAL_FLOAT(-3.14f, seni_strtof("-3.14", end));
  TEST_ASSERT_EQUAL_FLOAT(3.14f, seni_strtof(" 3.14", end));
  TEST_ASSERT_EQUAL_FLOAT(3.14f, seni_strtof(" 3.14  ", end));

  TEST_ASSERT_EQUAL_FLOAT(0.99f, seni_strtof(".99", end));
  TEST_ASSERT_EQUAL_FLOAT(15.0f, seni_strtof("15", end));
  TEST_ASSERT_EQUAL_FLOAT(0.0f, seni_strtof("0", end));
  TEST_ASSERT_EQUAL_FLOAT(1.0f, seni_strtof("1", end));
}

#define EVM_COMPILE(EXPR) seni_env *e = env_construct();                \
  seni_program *prog = program_compile(e, 256, EXPR);                   \
  seni_vm *vm = vm_construct(STACK_SIZE,HEAP_SIZE,HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES); \
  vm_debug_info_reset(vm);                                              \
  vm_interpret(vm, e, prog)

#define VM_TEST_FLOAT(RES) assert_seni_var_f32(stack_peek(vm), VAR_FLOAT, RES)
#define VM_TEST_BOOL(RES) assert_seni_var_bool(stack_peek(vm), RES)
#define VM_TEST_VEC4(A,B,C,D) assert_seni_var_v4(stack_peek(vm), A, B, C, D)
#define VM_TEST_VEC5(A,B,C,D,E) assert_seni_var_v5(stack_peek(vm), A, B, C, D, E)
#define VM_TEST_COL(F,A,B,C,D) assert_seni_var_col(stack_peek(vm), F, A, B, C, D)
#define VM_TEST_2D(A,B) assert_seni_var_2d(stack_peek(vm), A, B)

#define VM_CLEANUP program_free(prog);          \
  env_free(e);                                  \
  vm_free(vm)

// ************************************************
// TODO: use the above definition of VM_COMPILE_INT
// ************************************************
#define VM_COMPILE_F32(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_FLOAT(RES);VM_CLEANUP;}
#define VM_COMPILE_BOOL(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_BOOL(RES);VM_CLEANUP;}
#define VM_COMPILE_2D(EXPR,A,B) {EVM_COMPILE(EXPR);VM_TEST_2D(A,B);VM_CLEANUP;}
#define VM_COMPILE_VEC4(EXPR,A,B,C,D) {EVM_COMPILE(EXPR);VM_TEST_VEC4(A,B,C,D);VM_CLEANUP;}
#define VM_COMPILE_VEC5(EXPR,A,B,C,D,E) {EVM_COMPILE(EXPR);VM_TEST_VEC5(A,B,C,D,E);VM_CLEANUP;}
#define VM_COMPILE_COL(EXPR,F,A,B,C,D) {EVM_COMPILE(EXPR);VM_TEST_COL(F,A,B,C,D);VM_CLEANUP;}
// don't perform a heap check as we're assuming that the expr will be leaky
#define VM_COMPILE_F32_L(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_FLOAT(RES);VM_CLEANUP;}
#define VM_COMPILE_COL_L(EXPR,F,A,B,C,D) {EVM_COMPILE(EXPR);VM_TEST_COL(F,A,B,C,D);VM_CLEANUP;}

void timing(void)
{
  {
    TIMING_UNIT start = get_timing();
    //start = clock();
    //VM_COMPILE_F32("(step (x from: 0 to: 1000000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1)) 4", 4);

    VM_COMPILE_F32("(step (x from: 0 to: 10000) (step (y from: 0 to: 1000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (+ 3 4))) 9", 9);
    SENI_PRINT("VM Time taken %.2f", timing_delta_from(start));
  }
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

  VM_COMPILE_F32("(fn (f) (define rng (prng/build min: -1 max: 1 seed: 111)) (step (i from: 0 to: 2) (define [rr rx ry] (prng/values num: 3 from: rng)))) (f) 1", 1.0f);
  
  // pre-assigned global wasn't being added to the global-mapping so references to them in functions wasn't working
  VM_COMPILE_F32("(wash) (fn (wash) (define foo (/ canvas/width 3)) foo)", 333.3333f);

  // vm should use the caller function's ARG values not the callees.
  VM_COMPILE_F32("(fn (v foo: 10) foo) (fn (wash seed: 272) (v foo: seed)) (wash seed: 66)", 66.0f);

  // heap slab leak - overwriting local k in step
  // return vectors to slab when it's overwritten
  VM_COMPILE_F32("(fn (f) (step (i from: 0 to: 4) (define k [1 2])) 22)(f)", 22.0f);

  // return colours to slab when it's overwritten
  VM_COMPILE_F32("(fn (f) (step (i from: 0 to: 10) (define k (col/rgb r: 0 g: 0 b: 0 alpha: 1))) 22)(f)", 22.0f);

  // wasn't POP voiding function return values in a loop (CALL_0 offset was incorrect)
  // so have a loop that would overflow the stack if the return value of whatever fn wasn't being popped
  VM_COMPILE_F32("(fn (whatever))(fn (go)(define focalpoint (focal/build-point position: [0 0] distance: 100))(focal/value from: focalpoint position: [0 0])(step (y from: 0 to: 2000) (whatever))(focal/value from: focalpoint position: [0 50]))(go)", 0.5f);

}

void test_vm_bytecode(void)
{
  VM_COMPILE_F32("(define a 42) (define b 52) 10", 10);
  VM_COMPILE_F32("(define a 6) (define b 7) (+ a b)", 13);
  VM_COMPILE_F32("(define a 8 b 9) (+ a b)", 17);
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

  VM_COMPILE_F32("(step (x from: 0 to: 5) (+ 42 38)) 9", 9);
  VM_COMPILE_F32("(step (x from: 0 to: 5) (step (y from: 0 to: 5) (+ 3 4))) 9", 9);
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
  VM_COMPILE_F32("(fn (p2 a: 2) (+ a 5))(fn (p3 a: 3) (+ a 6))(fn (p4 a: 4) (+ a 7))(p2 a: (p3 a: (p4 a: 20)))", 38);

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

  // using a function before it's been declared
  VM_COMPILE_F32("(fn (x a: 33) (+ a (y c: 555))) (fn (y c: 444) c)  (x a: 66)", 621.0f);
  
  // passing an argument to a function that isn't being used
  // produces POP with VOID -1 args
  VM_COMPILE_F32("(fn (x a: 33) (+ a (y c: 555))) (fn (y c: 444) c)  (x a: 66 b: 8383)", 621.0f);
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

  // destructure a VAR_2D
  VM_COMPILE_F32("(fn (f pos: [3 5]) (define [j k] pos) (+ j k)) (f)", 8.0f);
  // destructure a VAR_VECTOR
  VM_COMPILE_F32("(fn (f pos: [3 5 7]) (define [j k l] pos) (+ j k l)) (f)", 15.0f);
}

void test_vm_2d(void)
{
  // constructing a VAR_2D
  VM_COMPILE_2D("(define vec2d [4 5]) vec2d", 4.0f, 5.0f);

  // destructuring works with VAR_2D
  VM_COMPILE_F32("(define [a b] [4 5]) (- b a)", 1.0f);

  // nth works with VAR_2D
  VM_COMPILE_F32("(define j [4 5]) (nth from: j n: 0)", 4.0f);
  VM_COMPILE_F32("(define j [4 5]) (nth from: j n: 1)", 5.0f);

  // READ_STACK_ARG_VEC2 in seni_bind.c
  VM_COMPILE_F32("(math/distance vec1: [0 3] vec2: [4 0])", 5.0f);

  
  VM_COMPILE_2D("[4 5]", 4, 5);

  // explicitly defined VAR_2D is returned
  VM_COMPILE_2D("(fn (f a: 3) [1 2]) (fn (x) (f)) (x)", 1, 2);

  // local var in function is returned
  VM_COMPILE_2D("(fn (f a: 3) (define b [1 2]) b) (fn (x) (f)) (x)", 1, 2);

  // local var in function is not returned
  VM_COMPILE_F32("(fn (f a: 3) (define b [1 2]) 55) (fn (x) (f)) (x)", 55);

  // default argument for function is returned
  VM_COMPILE_2D("(fn (f a: [1 2]) a) (fn (x) (f)) (x)", 1, 2);

  // default argument for function is not returned
  VM_COMPILE_F32("(fn (f a: [1 2]) 3) (fn (x) (f)) (x)", 3);

  // default argument for function is not returned and
  // it's called with an explicitly declared var_2d
  VM_COMPILE_F32("(fn (f a: [1 2]) 3) (fn (x) (f a: [3 4])) (x)", 3);

  // default argument for function is not returned and
  // it's called with an unused argument
  VM_COMPILE_F32("(fn (f a: [1 2]) 3) (fn (x) (f z: [3 4])) (x)", 3);

  // default argument for function is not returned
  VM_COMPILE_F32("(fn (f a: [1 2]) a) (fn (x) (f a: 5)) (x)", 5);
  
  // argument into function is returned
  VM_COMPILE_2D("(fn (f a: [3 4]) a) (fn (x) (f a: [1 2])) (x)", 1, 2);
}

void test_vm_vector(void)
{
  VM_COMPILE_VEC5("[4 5 6 7 8]", 4, 5, 6, 7, 8);

  VM_COMPILE_F32("(step (x from: 0 to: 5) [1 2 3 4 5]) 9", 9);

  // explicitly defined vector is returned
  VM_COMPILE_VEC5("(fn (f a: 3) [1 2 3 4 5]) (fn (x) (f)) (x)", 1, 2, 3, 4, 5);

  // local var in function is returned
  VM_COMPILE_VEC5("(fn (f a: 3) (define b [1 2 3 4 5]) b) (fn (x) (f)) (x)", 1, 2, 3, 4, 5);

  // local var in function is not returned
  VM_COMPILE_F32("(fn (f a: 3) (define b [1 2 3 4 5]) 55) (fn (x) (f)) (x)", 55);

  // default argument for function is returned
  VM_COMPILE_VEC5("(fn (f a: [1 2 3 4 5]) a) (fn (x) (f)) (x)", 1, 2, 3, 4, 5);

  // default argument for function is not returned
  VM_COMPILE_F32("(fn (f a: [1 2 3 4 5]) 3) (fn (x) (f)) (x)", 3);

  // default argument for function is not returned and
  // it's called with an explicitly declared vector
  VM_COMPILE_F32("(fn (f a: [1 2 3 4 5]) 3) (fn (x) (f a: [3 4])) (x)", 3);

  // default argument for function is not returned and
  // it's called with an unused argument
  VM_COMPILE_F32("(fn (f a: [1 2 3 4 5]) 3) (fn (x) (f z: [3 4])) (x)", 3);

  // default argument for function is not returned
  VM_COMPILE_F32("(fn (f a: [1 2 3 4 5]) a) (fn (x) (f a: 5)) (x)", 5);
  
  // argument into function is returned
  VM_COMPILE_VEC5("(fn (f a: [3 4 5 6 7]) a) (fn (x) (f a: [1 2 3 4 5])) (x)", 1, 2, 3, 4, 5);
}

void test_vm_vector_append(void)
{
  VM_COMPILE_F32("(define v []) (vector/append v 100) (vector/length vector: v)", 1);
  VM_COMPILE_F32("(define v [1]) (vector/append v 100) (vector/length vector: v)", 2);
  VM_COMPILE_F32("(define v [1 2]) (vector/append v 100) (vector/length vector: v)", 3);
  VM_COMPILE_F32("(define v [1 2 3]) (vector/append v 100) (vector/length vector: v)", 4);
  VM_COMPILE_F32("(define v [1 2 3 4]) (vector/append v 100) (vector/length vector: v)", 5);
}

void test_vm_fence(void)
{
  VM_COMPILE_F32("(define v []) (fence (x from: 0 to: 10 quantity: 3) (vector/append v x)) (vector/length vector: v)", 3);
  VM_COMPILE_F32("(define v []) (fence (x from: 0 to: 10 quantity: 3) (vector/append v x)) (nth from: v n: 0)", 0);
  VM_COMPILE_F32("(define v []) (fence (x from: 0 to: 10 quantity: 3) (vector/append v x)) (nth from: v n: 1)", 5);
  VM_COMPILE_F32("(define v []) (fence (x from: 0 to: 10 quantity: 3) (vector/append v x)) (nth from: v n: 2)", 10);

  VM_COMPILE_F32("(define v []) (fence (x quantity: 5) (vector/append v x)) (vector/length vector: v)", 5);
  VM_COMPILE_F32("(define v []) (fence (x quantity: 5) (vector/append v x)) (nth from: v n: 0)", 0.0f);
  VM_COMPILE_F32("(define v []) (fence (x quantity: 5) (vector/append v x)) (nth from: v n: 1)", 0.25f);
  VM_COMPILE_F32("(define v []) (fence (x quantity: 5) (vector/append v x)) (nth from: v n: 2)", 0.5f);
  VM_COMPILE_F32("(define v []) (fence (x quantity: 5) (vector/append v x)) (nth from: v n: 3)", 0.75f);
  VM_COMPILE_F32("(define v []) (fence (x quantity: 5) (vector/append v x)) (nth from: v n: 4)", 1.0f);
}

void test_vm_col_rgb(void)
{
  VM_COMPILE_COL_L("(col/rgb r: 0.1 g: 0.2 b: 0.3 alpha 0.4)", RGB, 0.1f, 0.2f, 0.3f, 0.4f);
  // checking colour_avail
  // TODO: this leaks colour, what should happen instead?
  // VM_COMPILE_F32("(fn (f) (col/rgb r: 0.1 g: 0.2 b: 0.3 alpha 0.4) 5) (f)", 5.0f);
}

void test_vm_math(void)
{
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

  VM_COMPILE_F32("(sqrt 144)", 12);
  
  VM_COMPILE_F32("(math/distance vec1: [0 0] vec2: [0 20])", 20.0f);
  VM_COMPILE_F32("(math/distance vec1: [0 5] vec2: [0 20])", 15.0f);
  VM_COMPILE_F32("(math/clamp value: 0.1 min: 0.0 max: 5)", 0.1f);
  VM_COMPILE_F32("(math/clamp value: -0.1 min: 0.0 max: 5)", 0.0f);
  VM_COMPILE_F32("(math/clamp value: 10 min: 0.0 max: 5)", 5.0f);
}

void test_vm_prng(void)
{
  // leaky because the global rng is a vector
  //
  VM_COMPILE_F32_L("(define rng (prng/build seed: 43215 min: 5 max: 20)) (prng/value from: rng)", 16.32348f);

  // state of rng is changing, returning a different number than previous tests
  VM_COMPILE_F32_L("(define rng (prng/build seed: 43215 min: 5 max: 20)) (prng/value from: rng) (prng/value from: rng)", 13.451335f);

  // wrapped in a function so that it's not leaky
  VM_COMPILE_F32("(fn (x) (define rng (prng/build seed: 43215 min: 5 max: 20)) (prng/value from: rng)) (x)", 16.32348f);

  // state of rng is changing, returning a different number than previous tests
  VM_COMPILE_F32("(fn (x) (define rng (prng/build seed: 43215 min: 5 max: 20)) (prng/value from: rng) (prng/value from: rng)) (x)", 13.451335f);

  // prng/take returning a vector
  VM_COMPILE_F32_L("(define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/values num: 3 from: rng)) (nth n: 0 from: foo)", 0.27065f);
  VM_COMPILE_F32_L("(define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/values num: 3 from: rng)) (nth n: 1 from: foo)", 0.75259f);
  VM_COMPILE_F32_L("(define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/values num: 3 from: rng)) (nth n: 2 from: foo)", -0.141389f);

  // non-leaky version of above
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/values num: 3 from: rng)) (nth n: 0 from: foo)) (x)", 0.27065f);
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/values num: 3 from: rng)) (nth n: 1 from: foo)) (x)", 0.75259f);
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo (prng/values num: 3 from: rng)) (nth n: 2 from: foo)) (x)", -0.141389f);

  // prng, destructuring, multiple args to '+'
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define [a b c] (prng/values num: 3 from: rng)) (+ a b c)) (x)", 0.881854f);
}

void test_vm_environmental(void)
{
  VM_COMPILE_F32("canvas/width", 1000.0f);
  VM_COMPILE_F32("canvas/height", 1000.0f);

}

void test_vm_interp(void)
{
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [0 1] to: [0 100])) (interp/value from: i t: 0.5)) (x)", 50.0f);

  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [10 20] to: [50 200])) (interp/value from: i t: 10.0)) (x)", 50.0f);
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [10 20] to: [50 200])) (interp/value from: i t: 20.0)) (x)", 200.0f);

  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [50 10] to: [100 1000])) (interp/value from: i t: 50.0)) (x)", 100.0f);
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [50 10] to: [100 1000])) (interp/value from: i t: 10.0)) (x)", 1000.0f);

  // clamping
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [0 1] to: [0 100] clamping: false)) (interp/value from: i t: 2.0)) (x)", 200.0f);
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [0 1] to: [0 100] clamping: true)) (interp/value from: i t: 2.0)) (x)", 100.0f);
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [0 1] to: [0 100] clamping: true)) (interp/value from: i t: -2.0)) (x)", 0.0f);
}

void test_vm_function_address(void)
{
  VM_COMPILE_F32("(fn (k a: 5) (+ a a)) (fn (l a: 5) (+ a a)) (define foo (address-of l)) (fn-call (foo a: 99 b: 88))", 198.0f);

  // normal
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo (address-of dbl)) (fn-call (foo a: 44))", 88.0f);
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo (address-of trp)) (fn-call (foo a: 44))", 132.0f);

  // invalid arguments - use defaults
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo (address-of dbl)) (fn-call (foo z: 44))", 10.0f);
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo (address-of trp)) (fn-call (foo z: 44))", 15.0f);

  // some invalid arguments
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo (address-of dbl)) (fn-call (foo z: 100 a: 44))", 88.0f);
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo (address-of trp)) (fn-call (foo z: 41 a: 44))", 132.0f);
  
}

void test_vm_repeat(void)
{
  // VM_COMPILE_F32("(fn (k) (+ 2 3)) (repeat/test draw: (address-of k)) 10", 10.0f);
  // VM_COMPILE_F32("(fn (k) [4 5]) (repeat/test draw: (address-of k)) 10", 10.0f);

  VM_COMPILE_F32("(fn (k a: 10 b: 20 c: 30) (+ a b c)) (k a: 40 b: 50 c: 60) 44", 44.0f);
}

void test_prng(void)
{

  seni_prng_state state;
  seni_prng_set_state(&state, 34342);
  
  f32 w;
  f32 min = 1000.0f;
  f32 max = -1000.0f;
  for (i32 i = 0; i < 1000000; i++) {
    w = seni_perlin(seni_prng_f32_range(&state, -100.0f, 100.0f),
                    seni_prng_f32_range(&state, -100.0f, 100.0f),
                    seni_prng_f32_range(&state, -100.0f, 100.0f));
    max = w > max ? w : max;
    min = w < min ? w : min;
  }

  printf("min %f, max %f\n", min, max);

  TEST_ASSERT_EQUAL_FLOAT(1.0f, seni_perlin(0.1f, 0.2f, 0.3f));
}

void unparse_compare(i32 seed_value, char *source, char *expected)
{
  seni_vm *vm = vm_construct(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  seni_env *env = env_construct();
  seni_shapes_init_globals();
  init_uv_mapper();

  seni_node *ast = parser_parse(env->wl, source);

  seni_trait_set *trait_set = trait_set_compile(ast, MAX_TRAIT_PROGRAM_SIZE, env->wl);

  // using the vm to build the genes
  seni_genotype *genotype = genotype_build(vm, env, trait_set, seed_value);

  i32 unparsed_source_size = 256;
  char *unparsed_source = (char *)calloc(unparsed_source_size, sizeof(char));
  unparse(unparsed_source, unparsed_source_size, env, ast, genotype);

  if (expected != NULL) {
    TEST_ASSERT_EQUAL_STRING(expected, unparsed_source);
  } else {
    TEST_ASSERT_EQUAL_STRING(source, unparsed_source);
  }


  free(unparsed_source);

  parser_free_nodes(ast);
  genotype_free(genotype);
  trait_set_free(trait_set);
  env_free(env);
  vm_free(vm);
  free_uv_mapper();
}


void test_unparser(void) {
  unparse_compare(9875, "(+ 4 2.0)", NULL);
  unparse_compare(9875, "(+ 4 [1 2 3])", NULL);
  unparse_compare(9875, "(+ 4 [1.0 2.22 3.333 4.4444 5.55555])", NULL);
  unparse_compare(9875, "red", NULL);
  unparse_compare(9875, "foo:", NULL);
  unparse_compare(9875, "foo ; some comment \"here\"", NULL);
  unparse_compare(9875, "(fn (a b: 10) (+ b 20))", NULL);
}

int main(void)
{
  // timing();

  if (INAME_NUMBER_OF_KNOWN_WORDS >= NATIVE_START) {
    SENI_LOG("WARNING: keywords are overwriting into NATIVE_START area");
  }
    
  UNITY_BEGIN();

  // RUN_TEST(debug_lang_interpret_mem); // for debugging/development
  // RUN_TEST(test_prng);
  // todo: test READ_STACK_ARG_COORD4


  // RUN_TEST(test_mathutil);
  // RUN_TEST(test_parser);
  // RUN_TEST(test_uv_mapper);
  // RUN_TEST(test_colour);
  // RUN_TEST(test_strtof);
  
  // // vm
  // RUN_TEST(test_vm_bugs);
  // RUN_TEST(test_vm_bytecode);
  // RUN_TEST(test_vm_callret);
  // RUN_TEST(test_vm_native);  
  // RUN_TEST(test_vm_destructure);
  // RUN_TEST(test_vm_2d);
  // RUN_TEST(test_vm_vector);
  // RUN_TEST(test_vm_vector_append);
  // RUN_TEST(test_vm_fence);
  // RUN_TEST(test_vm_col_rgb);
  // RUN_TEST(test_vm_math);
  // RUN_TEST(test_vm_prng);
  // RUN_TEST(test_vm_environmental);
  // RUN_TEST(test_vm_interp);
  // RUN_TEST(test_vm_function_address);
  // RUN_TEST(test_vm_repeat);

  RUN_TEST(test_unparser);

  return UNITY_END();
}
