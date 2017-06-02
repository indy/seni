/*
  Runs tests using the native compiler
*/
#include "unity/unity.h"
#include "seni.h"

#include "seni_lang.h"
#include "seni_bind.h"
#include "seni_uv_mapper.h"
#include "seni_colour.h"

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

  seni_uv_mapping *flat = get_uv_mapping(BRUSH_FLAT, 0);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.0f, flat->width_scale);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 2.0f / 1024.0f, flat->map[0].x);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.0f / 1024.0f, flat->map[0].y);
  
  TEST_ASSERT_NULL(get_uv_mapping(BRUSH_FLAT, 1)); // out of range

  seni_uv_mapping *c = get_uv_mapping(BRUSH_C, 8);
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

    assert_colour(rgb, clone_as(&res, rgb, RGB));
    assert_colour(hsl, clone_as(&res, rgb, HSL));
    assert_colour(lab, clone_as(&res, rgb, LAB));

    assert_colour(rgb, clone_as(&res, hsl, RGB));
    assert_colour(hsl, clone_as(&res, hsl, HSL));
    assert_colour(lab, clone_as(&res, hsl, LAB));

    assert_colour(rgb, clone_as(&res, lab, RGB));
    assert_colour(hsl, clone_as(&res, lab, HSL));
    assert_colour(lab, clone_as(&res, lab, LAB));

    colour_free(rgb);
    colour_free(hsl);
    colour_free(lab);
  }
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
  program_pretty_print(prog);

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
#define VM_HEAP_CHECK TEST_ASSERT_EQUAL_MESSAGE(vm->debug.get_from_heap_count, vm->debug.return_to_heap_count, "vm heap leak")
#else
#define VM_HEAP_CHECK
#endif

#define VM_TEST_FLOAT(RES) assert_seni_var_f32(stack_peek(vm), VAR_FLOAT, RES)
#define VM_TEST_BOOL(RES) assert_seni_var_bool(stack_peek(vm), RES)
#define VM_TEST_VEC2(A,B) assert_seni_var_v2(stack_peek(vm), A, B)
#define VM_TEST_VEC4(A,B,C,D) assert_seni_var_v4(stack_peek(vm), A, B, C, D)


#define VM_CLEANUP shutdown_interpreter_test(wl, ast);  \
  program_free(prog);                                   \
  env_free(e);                                          \
  vm_free(vm)


// COMPILE macros that eval and compare results
//
// 0 == print out bytecode, 1 == execute bytecode
#if 1
// ************************************************
// TODO: use the above definition of VM_COMPILE_INT
// ************************************************
#define VM_COMPILE_FLOAT(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_FLOAT(RES);VM_HEAP_CHECK;VM_CLEANUP;}
#define VM_COMPILE_BOOL(EXPR,RES) {EVM_COMPILE(EXPR);VM_TEST_BOOL(RES);VM_HEAP_CHECK;VM_CLEANUP;}
#define VM_COMPILE_VEC2(EXPR,A,B) {EVM_COMPILE(EXPR);VM_TEST_VEC2(A,B);VM_CLEANUP;}
#define VM_COMPILE_VEC4(EXPR,A,B,C,D) {EVM_COMPILE(EXPR);VM_TEST_VEC4(A,B,C,D);VM_CLEANUP;}

#else
// COMPILE macros that print out bytecode
//
#define VM_COMPILE_FLOAT(EXPR,_) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_BOOL(EXPR,_) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_VEC2(EXPR,_,_1) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#define VM_COMPILE_VEC4(EXPR,_,_1,_2,_3) {DVM_COMPILE(EXPR);VM_CLEANUP;}
#endif
// --------------------------------------------------

void timing(void)
{
  clock_t start, diff;
  int msec;
  {
    start = clock();
    //VM_COMPILE_FLOAT("(loop (x from: 0 to: 1000000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1)) 4", 4);

    VM_COMPILE_FLOAT("(loop (x from: 0 to: 10000) (loop (y from: 0 to: 1000) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (+ 3 4))) 9", 9);
    diff = clock() - start;
    msec = diff * 1000 / CLOCKS_PER_SEC;
    printf("VM Time taken %d seconds %d milliseconds\n", msec/1000, msec%1000);
  }
}

void test_vm_bytecode(void)
{
  VM_COMPILE_FLOAT("(define a 42) (define b 52) 10", 10);
  VM_COMPILE_FLOAT("(define a 6) (define b 7) (+ a b)", 13);
  VM_COMPILE_FLOAT("(+ 3 4)", 7);
  VM_COMPILE_FLOAT("(- (+ 1 2) 3)", 0);
  VM_COMPILE_FLOAT("(* 3 4)", 12);
  VM_COMPILE_FLOAT("(/ 90 10)", 9);
  VM_COMPILE_BOOL("(> 5 10)", false);
  VM_COMPILE_BOOL("(< 5 10)", true);
  VM_COMPILE_BOOL("(= 2 2)", true);
  VM_COMPILE_BOOL("(= 1 2)", false);
  VM_COMPILE_BOOL("(and (< 1 2) (< 3 4))", true);
  VM_COMPILE_BOOL("(and (< 1 2) (> 3 4))", false);
  VM_COMPILE_BOOL("(or (< 1 2) (> 3 4))", true);
  VM_COMPILE_BOOL("(not (> 1 10))", true);
  VM_COMPILE_BOOL("(and (or (< 1 2) (> 3 4)) (not (> 1 10)))", true);

  VM_COMPILE_FLOAT("(if (> 400 200) 66)", 66);
  VM_COMPILE_FLOAT("(if (> 200 100) 12 24)", 12);
  VM_COMPILE_FLOAT("(if (< 200 100) 12 24)", 24);
  VM_COMPILE_BOOL("(if (> 400 200) (= 50 50))", true);
  VM_COMPILE_BOOL("(if (> 99 88) (= 3 4) (= 5 5))", false);
  VM_COMPILE_BOOL("(if (< 99 88) (= 3 4) (= 5 5))", true);

  VM_COMPILE_FLOAT("(loop (x from: 0 to: 5) (+ 42 38)) 9", 9);
  VM_COMPILE_FLOAT("(loop (x from: 0 to: 5) (loop (y from: 0 to: 5) (+ 3 4))) 9", 9);
}

void test_vm_callret(void)
{
  // basic invocation
  VM_COMPILE_FLOAT("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: 3)", 8);
  VM_COMPILE_FLOAT("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: (+ 3 4))", 12); // calc required for value
  VM_COMPILE_FLOAT("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 xxx: 3)", 13); // non-existent argument
  VM_COMPILE_FLOAT("(fn (adder a: 9 b: 8) (+ a b)) (adder)", 17); // only default arguments
  VM_COMPILE_FLOAT("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 10)", 18); // missing argument
  VM_COMPILE_FLOAT("(fn (adder a: 9 b: 8) (+ a b)) (adder b: 20)", 29); // missing argument

  VM_COMPILE_FLOAT("(fn (p2 a: 1) (+ a 2)) (fn (p3 a: 1) (+ a 3)) (+ (p2 a: 5) (p3 a: 10))", 20);
  VM_COMPILE_FLOAT("(fn (p2 a: 1) (+ a 2)) (fn (p3 a: 1) (+ a 3)) (p2 a: (p3 a: 10))", 15);

  // functions calling functions
  VM_COMPILE_FLOAT("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z)))      (x)",       6);
  VM_COMPILE_FLOAT("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z a: 5))) (x)",      10);
  VM_COMPILE_FLOAT("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z a: 5))) (x c: 5)", 12);
}

void test_vm_vector(void)
{
  VM_COMPILE_VEC2("[4 5]", 4, 5);

  VM_COMPILE_FLOAT("(loop (x from: 0 to: 5) [1 2]) 9", 9);

  // explicitly defined vector is returned
  VM_COMPILE_VEC2("(fn (f a: 3) [1 2]) (fn (x) (f)) (x)", 1, 2);

  // local var in function is returned
  VM_COMPILE_VEC2("(fn (f a: 3) (define b [1 2]) b) (fn (x) (f)) (x)", 1, 2);

  // local var in function is not returned
  VM_COMPILE_FLOAT("(fn (f a: 3) (define b [1 2]) 55) (fn (x) (f)) (x)", 55);

  // default argument for function is returned
  VM_COMPILE_VEC2("(fn (f a: [1 2]) a) (fn (x) (f)) (x)", 1, 2);

  // default argument for function is not returned
  VM_COMPILE_FLOAT("(fn (f a: [1 2]) 3) (fn (x) (f)) (x)", 3);

  // default argument for function is not returned and
  // it's called with an explicitly declared vector
  VM_COMPILE_FLOAT("(fn (f a: [1 2]) 3) (fn (x) (f a: [3 4])) (x)", 3);

  // default argument for function is not returned and
  // it's called with an unused argument
  VM_COMPILE_FLOAT("(fn (f a: [1 2]) 3) (fn (x) (f z: [3 4])) (x)", 3);

  // default argument for function is not returned
  VM_COMPILE_FLOAT("(fn (f a: [1 2]) a) (fn (x) (f a: 5)) (x)", 5);
  
  // argument into function is returned
  VM_COMPILE_VEC2("(fn (f a: [3 4]) a) (fn (x) (f a: [1 2])) (x)", 1, 2);

}

void test_vm_col_rgb(void)
{
  // VM_COMPILE_VEC2("[4 5]", 4, 5);
  // VM_COMPILE_VEC4("[4 5 6 7]", 4, 5, 6, 7);

  VM_COMPILE_VEC4("(col/rgb r: 0.1 g: 0.2 b: 0.3 alpha 0.4)", 0.1f, 0.2f, 0.3f, 0.4f);
}

void test_temp(void)
{
  VM_COMPILE_FLOAT("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: 3)", 8);
  //VM_COMPILE_FLOAT("(fn (adder a: [1 2]) (+ 4 2)) (adder a: [4 5])", 8);

  // VM_COMPILE_FLOAT("(line width: 35 height: 22)", 17);
  // VM_COMPILE_FLOAT("(fn (adder a: 9 b: 8) (+ a b)) (line width: (+ 3 4) height: (adder))", 17);
  // VM_COMPILE_FLOAT("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: 3)", 8);
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
  RUN_TEST(test_vm_vector);
  RUN_TEST(test_vm_col_rgb);
  //RUN_TEST(test_temp);
  
  return UNITY_END();
}
