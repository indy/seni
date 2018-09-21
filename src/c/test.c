/*
  Runs tests using the native compiler
*/
#include "lib/unity/unity.h"

#include "sen/bind.h"
#include "sen/colour.h"
#include "sen/colour_scheme.h"
#include "sen/config.h"
#include "sen/cursor.h"
#include "sen/genetic.h"
#include "sen/keyword_iname.h"
#include "sen/lang.h"
#include "sen/lib.h"
#include "sen/mathutil.h"
#include "sen/multistring.h"
#include "sen/parser.h"
#include "sen/pool_macro.h"
#include "sen/prng.h"
#include "sen/shapes.h"
#include "sen/strtof.h"
#include "sen/timing.h"
#include "sen/types.h"
#include "sen/unparser.h"
#include "sen/uv_mapper.h"
#include "sen/vm_compiler.h"
#include "sen/vm_interpreter.h"

#include "stdio.h"
#include <stdlib.h>
#include <string.h>

#include <stdarg.h>

#include "lib/utlist.h"

/* way of working with boolean and TEST macros */
bool test_true  = true;
bool test_false = false;

/* required by unity */
void setUp(void) {}
void tearDown(void) {}

// testing the pool macro

struct sen_item {
  i32 id;

  struct sen_item* next;
  struct sen_item* prev;
};
typedef struct sen_item sen_item;

void item_cleanup(sen_item* item) { item->id = 0; }

SEN_POOL(sen_item, item)

void test_macro_pool(void) {
  sen_item*             item;
  struct sen_item_pool* item_pool;
  i32                   i;

  {
    item      = NULL;
    item_pool = item_pool_allocate(1, 10, 2);

    TEST_ASSERT_EQUAL(item_pool->num_slabs, 1);
    TEST_ASSERT_EQUAL(item_pool->max_slabs_allowed, 2);
    TEST_ASSERT_EQUAL(item_pool->high_water_mark, 0);
    TEST_ASSERT_EQUAL(item_pool->current_water_mark, 0);

    item = item_pool_get(item_pool);
    TEST_ASSERT_EQUAL(item_pool->get_count, 1);
    TEST_ASSERT_EQUAL(item_pool->return_count, 0);
    TEST_ASSERT_EQUAL(item_pool->high_water_mark, 1);
    TEST_ASSERT_EQUAL(item_pool->current_water_mark, 1);

    item = item_pool_get(item_pool);
    TEST_ASSERT_EQUAL(item_pool->get_count, 2);
    TEST_ASSERT_EQUAL(item_pool->return_count, 0);
    TEST_ASSERT_EQUAL(item_pool->high_water_mark, 2);
    TEST_ASSERT_EQUAL(item_pool->current_water_mark, 2);

    item_pool_return(item_pool, item);
    TEST_ASSERT_EQUAL(item_pool->get_count, 2);
    TEST_ASSERT_EQUAL(item_pool->return_count, 1);
    TEST_ASSERT_EQUAL(item_pool->high_water_mark, 2);
    TEST_ASSERT_EQUAL(item_pool->current_water_mark, 1);

    // get enough sen_items to allocate a new slab
    for (i = 0; i < 15; i++) {
      item = item_pool_get(item_pool);
    }
    TEST_ASSERT_EQUAL(item_pool->high_water_mark, 16);
    TEST_ASSERT_EQUAL(item_pool->current_water_mark, 16);

    TEST_ASSERT_EQUAL(item_pool->num_slabs, 2);

    item_pool_free(item_pool);
  }

  {
    item      = NULL;
    item_pool = item_pool_allocate(1, 10, 2);
    // repeatedly allocate and return a sen_item
    for (i = 0; i < 150; i++) {
      item = item_pool_get(item_pool);
      item_pool_return(item_pool, item);
    }
    // should still only be 1 slab allocated
    TEST_ASSERT_EQUAL(item_pool->num_slabs, 1);

    TEST_ASSERT_EQUAL(item_pool->high_water_mark, 1);
    TEST_ASSERT_EQUAL(item_pool->current_water_mark, 0);

    item_pool_free(item_pool);
  }

  {
    item      = NULL;
    item_pool = item_pool_allocate(1, 10, 2);
    i32       j;
    sen_item* items[10];

    // repeatedly allocate and return sets of sen_items
    for (i = 0; i < 50; i++) {
      for (j = 0; j < 10; j++) {
        items[j] = item_pool_get(item_pool);
      }
      for (j = 0; j < 10; j++) {
        item_pool_return(item_pool, items[j]);
      }
    }
    // should still only be 1 slab allocated
    TEST_ASSERT_EQUAL(item_pool->num_slabs, 1);

    TEST_ASSERT_EQUAL(item_pool->high_water_mark, 10);
    TEST_ASSERT_EQUAL(item_pool->current_water_mark, 0);

    item_pool_free(item_pool);
  }
}

void test_mathutil(void) {
  TEST_ASSERT_EQUAL_FLOAT(1.5f, deg_to_rad(rad_to_deg(1.5f)));
  TEST_ASSERT_EQUAL_FLOAT(0.44444f, mc_m(1.0f, 1.0f, 10.0f, 5.0f));
  TEST_ASSERT_EQUAL_FLOAT(0.55556f, mc_c(1.0f, 1.0f, 0.444444f));
}

sen_node* assert_parser_node_raw(sen_node* node, sen_node_type type) {
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));
  return node->next;
}

sen_node* assert_parser_node_i32(sen_node* node, sen_node_type type, i32 val) {
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));
  TEST_ASSERT_EQUAL(val, node->value.i);
  return node->next;
}

sen_node* assert_parser_node_f32(sen_node* node, f32 val) {
  TEST_ASSERT_EQUAL_MESSAGE(NODE_FLOAT, node->type, node_type_name(node));
  TEST_ASSERT_EQUAL_FLOAT(val, node->value.f);
  return node->next;
}

sen_node* assert_parser_node_str(sen_node* node, sen_node_type type, char* val) {
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));

  i32   count    = 0;
  char* expected = val;
  char* actual   = node->src;
  while (*expected != '\0') {
    count++;
    TEST_ASSERT_EQUAL(*expected, *actual);
    expected++;
    actual++;
  }

  TEST_ASSERT_EQUAL(count, node->src_len);

  return node->next;
}

sen_node*
assert_parser_node_txt(sen_node* node, sen_node_type type, char* val, sen_word_lut* word_lut) {
  TEST_ASSERT_EQUAL_MESSAGE(type, node->type, node_type_name(node));

  i32 i = node->value.i;
  TEST_ASSERT_TRUE(word_lut->word_count > i);

  sen_string_ref* string_ref = &(word_lut->word_ref[i]);
  TEST_ASSERT_EQUAL_STRING(val, string_ref->c);

  return node->next;
}

#define PARSE(EXPR)           \
  parser_subsystem_startup(); \
  word_lut = wlut_allocate(); \
  nodes    = parser_parse(word_lut, EXPR)

#define PARSE_CLEANUP                 \
  wlut_free(word_lut);                \
  parser_return_nodes_to_pool(nodes); \
  parser_subsystem_shutdown();

void test_parser(void) {
  sen_node *    nodes, *iter, *iter2;
  sen_word_lut* word_lut;

  PARSE("hello");
  assert_parser_node_txt(nodes, NODE_NAME, "hello", word_lut);
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
  iter = assert_parser_node_txt(iter, NODE_NAME, "add", word_lut);
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
  iter = assert_parser_node_txt(iter, NODE_NAME, "add", word_lut);
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
  iter  = nodes->value.first_child;
  iter  = assert_parser_node_txt(iter, NODE_NAME, "quote", word_lut);
  iter  = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter;
  iter  = assert_parser_node_raw(iter, NODE_LIST);
  TEST_ASSERT_NULL(iter);
  iter = iter2->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "runall", word_lut);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_STRING, "shabba", word_lut);
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_str(iter, NODE_COMMENT, "; woohoo");
  TEST_ASSERT_NULL(iter);
  PARSE_CLEANUP;

  PARSE("(fun i: 42 f: 12.34)");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "fun", word_lut);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_LABEL, "i", word_lut);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 42);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_txt(iter, NODE_LABEL, "f", word_lut);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 12.34f);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;

  PARSE("(a 1) (b 2)");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter = nodes->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "a", word_lut);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 1);
  TEST_ASSERT_NULL(iter);
  iter = nodes->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  assert_parser_node_raw(iter, NODE_LIST);
  iter = iter->value.first_child;
  iter = assert_parser_node_txt(iter, NODE_NAME, "b", word_lut);
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter = assert_parser_node_f32(iter, 2);
  TEST_ASSERT_NULL(iter);
  PARSE_CLEANUP;

  PARSE("[[0.1 0.2] [0.3 0.4]]");
  assert_parser_node_raw(nodes, NODE_VECTOR); // outer []
  iter = nodes->value.first_child;
  assert_parser_node_raw(iter, NODE_VECTOR); // [0.1 0.2]
  iter2 = iter->value.first_child;
  iter2 = assert_parser_node_f32(iter2, 0.1f);
  iter2 = assert_parser_node_str(iter2, NODE_WHITESPACE, " ");
  iter2 = assert_parser_node_f32(iter2, 0.2f);
  TEST_ASSERT_NULL(iter2);
  iter = iter->next;
  iter = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  assert_parser_node_raw(iter, NODE_VECTOR); // [0.3 0.4]
  iter2 = iter->value.first_child;
  iter2 = assert_parser_node_f32(iter2, 0.3f);
  iter2 = assert_parser_node_str(iter2, NODE_WHITESPACE, " ");
  iter2 = assert_parser_node_f32(iter2, 0.4f);
  TEST_ASSERT_NULL(iter2);
  TEST_ASSERT_NULL(iter->next);
  PARSE_CLEANUP;

  PARSE("(a {[1 2]})");
  assert_parser_node_raw(nodes, NODE_LIST);
  iter  = nodes->value.first_child;
  iter  = assert_parser_node_txt(iter, NODE_NAME, "a", word_lut);
  iter  = assert_parser_node_str(iter, NODE_WHITESPACE, " ");
  iter2 = iter; // the vector
  iter  = assert_parser_node_raw(iter, NODE_VECTOR);
  TEST_ASSERT_NULL(iter);
  TEST_ASSERT_EQUAL(test_true, iter2->alterable);
  TEST_ASSERT_NULL(nodes->next);
  PARSE_CLEANUP;
}

void assert_sen_var_f32(sen_var* var, sen_var_type type, f32 f) {
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var));
  TEST_ASSERT_EQUAL_FLOAT(f, var->value.f);
}

void assert_sen_var_v4(sen_var* var, f32 a, f32 b, f32 c, f32 d) {
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VECTOR, var->type, "VAR_VECTOR");

  sen_var* val = var->value.v;
  TEST_ASSERT_EQUAL_FLOAT(a, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(b, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(c, val->value.f);

  val = val->next;
  TEST_ASSERT_EQUAL_FLOAT(d, val->value.f);
}

void assert_sen_var_v5(sen_var* var, f32 a, f32 b, f32 c, f32 d, f32 e) {
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VECTOR, var->type, "VAR_VECTOR");

  sen_var* val = var->value.v;
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

void assert_sen_var_col(sen_var* var, i32 format, f32 a, f32 b, f32 c, f32 d) {
  TEST_ASSERT_EQUAL_MESSAGE(VAR_COLOUR, var->type, "VAR_COLOUR");

  // sen_colour *colour = var->value.c;

  TEST_ASSERT_EQUAL(format, (i32)var->value.i);

  TEST_ASSERT_FLOAT_WITHIN(0.1f, a, var->f32_array[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, b, var->f32_array[1]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, c, var->f32_array[2]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, d, var->f32_array[3]);
}

void assert_sen_var_2d(sen_var* var, f32 a, f32 b) {
  TEST_ASSERT_EQUAL_MESSAGE(VAR_2D, var->type, "VAR_2D");

  TEST_ASSERT_FLOAT_WITHIN(0.1f, a, var->f32_array[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, b, var->f32_array[1]);
}

void assert_sen_var_f32_within(sen_var* var, sen_var_type type, f32 f, f32 tolerance) {
  TEST_ASSERT_EQUAL_MESSAGE(type, var->type, var_type_name(var));
  TEST_ASSERT_FLOAT_WITHIN(tolerance, f, var->value.f);
}

void assert_sen_var_bool(sen_var* var, bool b) {
  TEST_ASSERT_EQUAL_MESSAGE(VAR_BOOLEAN, var->type, var_type_name(var));
  TEST_ASSERT_EQUAL(b ? 1 : 0, var->value.i);
}

void test_uv_mapper(void) {
  uv_mapper_subsystem_startup();

  sen_uv_mapping* flat = get_uv_mapping(BRUSH_FLAT, 0, true);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.0f, flat->width_scale);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 2.0f / 1024.0f, flat->map[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.0f / 1024.0f, flat->map[1]);

  TEST_ASSERT_NULL(get_uv_mapping(BRUSH_FLAT, 1, false)); // out of range

  sen_uv_mapping* c = get_uv_mapping(BRUSH_C, 8, false);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 1.1f, c->width_scale);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 326.0f / 1024.0f, c->map[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, 556.0f / 1024.0f, c->map[1]);

  uv_mapper_subsystem_shutdown();
}

// temp printing: DELETE THIS
void colour_print(char* msg, sen_colour* colour) {
  SEN_PRINT("%s: %d [%.4f, %.4f, %.4f]",
            msg,
            colour->format,
            colour->element[0],
            colour->element[1],
            colour->element[2]);
}

void assert_colour(sen_colour* expected, sen_colour* colour) {
  // colour_print("in assert", colour);
  TEST_ASSERT_EQUAL(expected->format, colour->format);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, expected->element[0], colour->element[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, expected->element[1], colour->element[1]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, expected->element[2], colour->element[2]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, expected->element[3], colour->element[3]);
}

void assert_colour_e(sen_colour*       colour,
                     sen_colour_format format,
                     f32               e0,
                     f32               e1,
                     f32               e2,
                     f32               alpha) {
  TEST_ASSERT_EQUAL(format, colour->format);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, e0, colour->element[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, e1, colour->element[1]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, e2, colour->element[2]);
  TEST_ASSERT_FLOAT_WITHIN(0.1f, alpha, colour->element[3]);
}

void colour_def(sen_colour* out, sen_colour_format format, f32 e0, f32 e1, f32 e2, f32 alpha) {
  out->format     = format;
  out->element[0] = e0;
  out->element[1] = e1;
  out->element[2] = e2;
  out->element[3] = alpha;
}

void assert_colour_rgb_hsl(f32 r, f32 g, f32 b, f32 h, f32 s, f32 l) {
  sen_colour rgb, hsl, res;

  colour_def(&rgb, RGB, r, g, b, 1.0f);
  colour_def(&hsl, HSL, h, s, l, 1.0f);

  // convert RGB to HSL
  colour_clone_as(&res, &rgb, HSL);
  assert_colour_e(&res, HSL, h, s, l, 1.0f);

  // convert HSL to RGB
  colour_clone_as(&res, &hsl, RGB);
  assert_colour_e(&res, RGB, r, g, b, 1.0f);
}

void test_colour(void) {
  sen_colour c, rgb, hsl, lab, hsluv, res;

  {
    colour_def(&c, RGB, 0.0f, 0.0f, 0.0f, 1.0f);
    TEST_ASSERT_EQUAL(RGB, c.format);
    TEST_ASSERT_EQUAL_FLOAT(0.0f, c.element[0]);
    TEST_ASSERT_EQUAL_FLOAT(0.0f, c.element[1]);
    TEST_ASSERT_EQUAL_FLOAT(0.0f, c.element[2]);
    TEST_ASSERT_EQUAL_FLOAT(1.0f, c.element[3]);
  }

  {
    colour_def(&rgb, RGB, 0.2f, 0.09803921568627451f, 0.49019607843137253f,
               1.0f); // (51, 25, 125)
    colour_def(&hsl, HSL, 255.6f, 0.6666f, 0.294f, 1.0f);
    colour_def(&lab, LAB, 19.555676428108306f, 39.130689315704764f, -51.76254071703564f, 1.0f);

    assert_colour(&rgb, colour_clone_as(&res, &rgb, RGB));
    assert_colour(&hsl, colour_clone_as(&res, &rgb, HSL));
    assert_colour(&lab, colour_clone_as(&res, &rgb, LAB));

    assert_colour(&rgb, colour_clone_as(&res, &hsl, RGB));
    assert_colour(&hsl, colour_clone_as(&res, &hsl, HSL));
    assert_colour(&lab, colour_clone_as(&res, &hsl, LAB));

    assert_colour(&rgb, colour_clone_as(&res, &lab, RGB));
    assert_colour(&hsl, colour_clone_as(&res, &lab, HSL));
    assert_colour(&lab, colour_clone_as(&res, &lab, LAB));
  }

  {
    colour_def(&rgb, RGB, 0.066666f, 0.8f, 0.86666666f, 1.0f);
    colour_def(&hsluv, HSLuv, 205.7022764106217f, 98.91247496876854f, 75.15356872935901f, 1.0f);
    assert_colour(&rgb, colour_clone_as(&res, &hsluv, RGB));
    assert_colour(&hsluv, colour_clone_as(&res, &rgb, HSLuv));
  }

  // hsl <-> rgb
  {
    assert_colour_rgb_hsl(0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f);
    assert_colour_rgb_hsl(1.0f, 1.0f, 1.0f, 0.0f, 0.0f, 1.0f);
    assert_colour_rgb_hsl(1.0f, 0.0f, 0.0f, 0.0f, 1.0f, 0.5f);
    assert_colour_rgb_hsl(0.0f, 1.0f, 0.0f, 120.0f, 1.0f, 0.5f);
    assert_colour_rgb_hsl(0.0f, 0.0f, 1.0f, 240.0f, 1.0f, 0.5f);
    assert_colour_rgb_hsl(1.0f, 1.0f, 0.0f, 60.0f, 1.0f, 0.5f);
    assert_colour_rgb_hsl(0.0f, 1.0f, 1.0f, 180.0f, 1.0f, 0.5f);
    assert_colour_rgb_hsl(1.0f, 0.0f, 1.0f, 300.0f, 1.0f, 0.5f);
    assert_colour_rgb_hsl(0.7529f, 0.7529f, 0.7529f, 0.0f, 0.0f, 0.75f);
    assert_colour_rgb_hsl(0.5f, 0.5f, 0.5f, 0.0f, 0.0f, 0.5f);
    assert_colour_rgb_hsl(0.5f, 0.0f, 0.0f, 0.0f, 1.0f, 0.25f);
    assert_colour_rgb_hsl(0.5f, 0.5f, 0.0f, 60.0f, 1.0f, 0.25f);
    assert_colour_rgb_hsl(0.0f, 0.5f, 0.0f, 120.0f, 1.0f, 0.25f);
    assert_colour_rgb_hsl(0.5f, 0.0f, 0.5f, 300.0f, 1.0f, 0.25f);
    assert_colour_rgb_hsl(0.0f, 0.5f, 0.5f, 180.0f, 1.0f, 0.25f);
    assert_colour_rgb_hsl(0.0f, 0.0f, 0.5f, 240.0f, 1.0f, 0.25f);
  }
}

void test_strtof(void) {
  char** end = NULL;

  TEST_ASSERT_EQUAL_FLOAT(3.14f, sen_strtof("3.14", end));
  TEST_ASSERT_EQUAL_FLOAT(-3.14f, sen_strtof("-3.14", end));
  TEST_ASSERT_EQUAL_FLOAT(3.14f, sen_strtof(" 3.14", end));
  TEST_ASSERT_EQUAL_FLOAT(3.14f, sen_strtof(" 3.14  ", end));

  TEST_ASSERT_EQUAL_FLOAT(0.99f, sen_strtof(".99", end));
  TEST_ASSERT_EQUAL_FLOAT(15.0f, sen_strtof("15", end));
  TEST_ASSERT_EQUAL_FLOAT(0.0f, sen_strtof("0", end));
  TEST_ASSERT_EQUAL_FLOAT(1.0f, sen_strtof("1", end));
}

#define VM_COMPILE(EXPR)                                                                             \
  sen_systems_startup();                                                                             \
  sen_env*     e    = env_allocate();                                                                \
  sen_program* prog = sen_compile_program(EXPR, e->word_lut, 256);                                   \
  sen_vm*      vm   = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES); \
  vm_debug_info_reset(vm);                                                                           \
  vm_run(vm, e, prog)

#define VM_TEST_FLOAT(RES) assert_sen_var_f32(vm_stack_peek(vm), VAR_FLOAT, RES)
#define VM_TEST_BOOL(RES) assert_sen_var_bool(vm_stack_peek(vm), RES)
#define VM_TEST_VEC4(A, B, C, D) assert_sen_var_v4(vm_stack_peek(vm), A, B, C, D)
#define VM_TEST_VEC5(A, B, C, D, E) assert_sen_var_v5(vm_stack_peek(vm), A, B, C, D, E)
#define VM_TEST_COL(F, A, B, C, D) assert_sen_var_col(vm_stack_peek(vm), F, A, B, C, D)
#define VM_TEST_2D(A, B) assert_sen_var_2d(vm_stack_peek(vm), A, B)

#define VM_CLEANUP    \
  program_free(prog); \
  env_free(e);        \
  vm_free(vm);        \
  sen_systems_shutdown();

#define VM_COMPILE_F32(EXPR, RES) \
  {                               \
    VM_COMPILE(EXPR);             \
    VM_TEST_FLOAT(RES);           \
    VM_CLEANUP;                   \
  }
#define VM_COMPILE_BOOL(EXPR, RES) \
  {                                \
    VM_COMPILE(EXPR);              \
    VM_TEST_BOOL(RES);             \
    VM_CLEANUP;                    \
  }
#define VM_COMPILE_2D(EXPR, A, B) \
  {                               \
    VM_COMPILE(EXPR);             \
    VM_TEST_2D(A, B);             \
    VM_CLEANUP;                   \
  }
#define VM_COMPILE_VEC4(EXPR, A, B, C, D) \
  {                                       \
    VM_COMPILE(EXPR);                     \
    VM_TEST_VEC4(A, B, C, D);             \
    VM_CLEANUP;                           \
  }
#define VM_COMPILE_VEC5(EXPR, A, B, C, D, E) \
  {                                          \
    VM_COMPILE(EXPR);                        \
    VM_TEST_VEC5(A, B, C, D, E);             \
    VM_CLEANUP;                              \
  }
#define VM_COMPILE_COL(EXPR, F, A, B, C, D) \
  {                                         \
    VM_COMPILE(EXPR);                       \
    VM_TEST_COL(F, A, B, C, D);             \
    VM_CLEANUP;                             \
  }
// don't perform a heap check as we're assuming that the expr will be leaky
#define VM_COMPILE_F32_L(EXPR, RES) \
  {                                 \
    VM_COMPILE(EXPR);               \
    VM_TEST_FLOAT(RES);             \
    VM_CLEANUP;                     \
  }
#define VM_COMPILE_COL_L(EXPR, F, A, B, C, D) \
  {                                           \
    VM_COMPILE(EXPR);                         \
    VM_TEST_COL(F, A, B, C, D);               \
    VM_CLEANUP;                               \
  }

void f32v(char* exp, i32 len, ...) {
  VM_COMPILE(exp);

  sen_var* var = vm_stack_peek(vm);
  TEST_ASSERT_EQUAL_MESSAGE(VAR_VECTOR, var->type, "f32v: VAR_VECTOR");
  TEST_ASSERT_EQUAL(len, vector_length(var));

  va_list va;
  va_start(va, len);

  f32      val;
  sen_var* element;
  for (i32 i = 0; i < len; i++) {
    element = vector_get(var, i);
    TEST_ASSERT_EQUAL_MESSAGE(VAR_FLOAT, element->type, "f32v: VAR_FLOAT");

    val = (f32)va_arg(va, double);
    TEST_ASSERT_EQUAL_FLOAT(val, element->value.f);
  }

  va_end(va);

  VM_CLEANUP;
}

void timing(void) {
  {
    TIMING_UNIT start = get_timing();
    // start = clock();
    // VM_COMPILE_F32("(loop (x from: 0 to: 1000000) (- 1 1) (- 1 1) (- 1 1) (-
    // 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1
    // 1) (- 1 1) (- 1 1) (- 1 1)) 4", 4);

    VM_COMPILE_F32("(loop (x from: 0 to: 10000) (loop (y from: 0 to: 1000) (- 1 1) (- 1 "
                   "1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (- 1 1) (+ 3 4))) 9",
                   9);
    SEN_PRINT("VM Time taken %.2f", timing_delta_from(start));
  }
}

// code that exposed bugs - but was later fixed
void test_vm_bugs(void) {
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
   (huh at: 5)",
                 4.0f);

  VM_COMPILE_F32("(fn (f) (define rng (prng/build min: -1 max: 1 seed: 111)) "
                 "(loop (i from: 0 to: 2) (define [rr rx ry] (prng/values num: "
                 "3 from: rng)))) (f) 1",
                 1.0f);

  // pre-assigned global wasn't being added to the global-mapping so references
  // to them in functions wasn't working
  VM_COMPILE_F32("(wash) (fn (wash) (define foo (/ canvas/width 3)) foo)", 333.3333f);

  // vm should use the caller function's ARG values not the callees.
  VM_COMPILE_F32("(fn (v foo: 10) foo) (fn (wash seed: 272) (v foo: seed)) "
                 "(wash seed: 66)",
                 66.0f);

  // heap slab leak - overwriting local k in loop
  // return vectors to slab when it's overwritten
  VM_COMPILE_F32("(fn (f) (loop (i from: 0 to: 4) (define k [1 2])) 22)(f)", 22.0f);

  // return colours to slab when it's overwritten
  VM_COMPILE_F32("(fn (f) (loop (i from: 0 to: 10) (define k (col/rgb r: 0 g: "
                 "0 b: 0 alpha: 1))) 22)(f)",
                 22.0f);

  // wasn't POP voiding function return values in a loop (CALL_0 offset was
  // incorrect) so have a loop that would overflow the stack if the return value
  // of whatever fn wasn't being popped
  VM_COMPILE_F32("(fn (whatever))(fn (go)(define focalpoint (focal/build-point position: "
                 "[0 0] distance: 100))(focal/value from: focalpoint position: [0 "
                 "0])(loop (y from: 0 to: 2000) (whatever))(focal/value from: focalpoint "
                 "position: [0 50]))(go)",
                 0.5f);
}

void test_vm_bytecode(void) {
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

  VM_COMPILE_F32("(loop (x from: 0 to: 5) (+ 42 38)) 9", 9);
  VM_COMPILE_F32("(loop (x from: 0 to: 5) (loop (y from: 0 to: 5) (+ 3 4))) 9", 9);
}

void test_vm_callret(void) {
  // basic invocation
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: 3)", 8);
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 b: (+ 3 4))",
                 12); // calc required for value
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 5 xxx: 3)",
                 13); // non-existent argument
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder)",
                 17); // only default arguments
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder a: 10)",
                 18); // missing argument
  VM_COMPILE_F32("(fn (adder a: 9 b: 8) (+ a b)) (adder b: 20)",
                 29); // missing argument

  VM_COMPILE_F32("(fn (p2 a: 1) (+ a 2)) (fn (p3 a: 1) (+ a 3)) (+ (p2 a: 5) (p3 a: 10))", 20);
  VM_COMPILE_F32("(fn (p2 a: 1) (+ a 2)) (fn (p3 a: 1) (+ a 3)) (p2 a: (p3 a: 10))", 15);
  VM_COMPILE_F32("(fn (p2 a: 2) (+ a 5))(fn (p3 a: 3) (+ a 6))(fn (p4 a: 4) (+ "
                 "a 7))(p2 a: (p3 a: (p4 a: 20)))",
                 38);

  // functions calling functions
  VM_COMPILE_F32("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z)))      (x)", 6);
  VM_COMPILE_F32("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z a: 5))) (x)", 10);
  VM_COMPILE_F32("(fn (z a: 1) (+ a 2)) (fn (x c: 3) (+ c (z a: 5))) (x c: 5)", 12);

  // function calling another function, passing on one of it's local variables
  // (make use of the hop_back method of referring to the correct LOCAL frame)
  VM_COMPILE_F32("(fn (z a: 1) (+ a 5)) (fn (y) (define x 10) (z a: x)) (y)", 15);
  VM_COMPILE_F32("(fn (z a: 1) (+ a 5)) (fn (zz a: 1) (+ a 9))(fn (y) (define "
                 "x 10) (z a: (zz a: x))) (y)",
                 24);

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

void test_vm_native(void) {
  // call native functions that have vector arguments (tests ref counting)
  VM_COMPILE_F32("(nth n: 0 from: [22 33])", 22);
  VM_COMPILE_F32_L("(define aa [22 33]) (nth n: 0 from: aa)", 22);
  VM_COMPILE_F32("(fn (x) (nth n: 0 from: [22 33])) (x)", 22);
  VM_COMPILE_F32("(math/distance vec1: [0 0] vec2: [0 20])", 20.0f);
}

void test_vm_destructure(void) {
  VM_COMPILE_F32("(define [a b] [22 33]) (- b a)", 11);
  VM_COMPILE_F32("(define [a b c] [22 33 44]) (+ a b c)", 99);

  // destructure a VAR_2D
  VM_COMPILE_F32("(fn (f pos: [3 5]) (define [j k] pos) (+ j k)) (f)", 8.0f);
  // destructure a VAR_VECTOR
  VM_COMPILE_F32("(fn (f pos: [3 5 7]) (define [j k l] pos) (+ j k l)) (f)", 15.0f);
}

void test_vm_2d(void) {
  // constructing a VAR_2D
  VM_COMPILE_2D("(define vec2d [4 5]) vec2d", 4.0f, 5.0f);

  // destructuring works with VAR_2D
  VM_COMPILE_F32("(define [a b] [4 5]) (- b a)", 1.0f);

  // nth works with VAR_2D
  VM_COMPILE_F32("(define j [4 5]) (nth from: j n: 0)", 4.0f);
  VM_COMPILE_F32("(define j [4 5]) (nth from: j n: 1)", 5.0f);

  // READ_STACK_ARG_VEC2 in sen_bind.c
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

void test_vm_vector(void) {
  VM_COMPILE_VEC5("[4 5 6 7 8]", 4, 5, 6, 7, 8);

  VM_COMPILE_F32("(loop (x from: 0 to: 5) [1 2 3 4 5]) 9", 9);

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

void test_vm_vector_append(void) {
  VM_COMPILE_F32("(define v []) (vector/append v 100) (vector/length vector: v)", 1);
  VM_COMPILE_F32("(define v [1]) (vector/append v 100) (vector/length vector: v)", 2);
  VM_COMPILE_F32("(define v [1 2]) (vector/append v 100) (vector/length vector: v)", 3);
  VM_COMPILE_F32("(define v [1 2 3]) (vector/append v 100) (vector/length vector: v)", 4);
  VM_COMPILE_F32("(define v [1 2 3 4]) (vector/append v 100) (vector/length vector: v)", 5);
}

void test_vm_fence(void) {
  f32v("(define v []) (fence (x from: 0 to: 10 num: 3) (vector/append v x)) v",
       3,
       0.0f,
       5.0f,
       10.0f);
  f32v("(define v []) (fence (x from: 10 to: 0 num: 3) (vector/append v x)) v",
       3,
       10.0f,
       5.0f,
       0.0f);
  f32v("(define v []) (fence (x num: 5) (vector/append v x)) v", 5, 0.0f, 0.25f, 0.5f, 0.75f, 1.0f);
  f32v("(define v []) (fence (x from: 100 to: 900 num: 10) (vector/append v "
       "x)) v",
       10,
       100.0000f,
       188.8889f,
       277.7778f,
       366.6667f,
       455.5555f,
       544.4445f,
       633.3333f,
       722.2222f,
       811.1111f,
       900.0000f);
}

void test_vm_loop(void) {
  f32v("(define v []) (loop (x from: 0 to: 4) (vector/append v x)) v", 4, 0.0f, 1.0f, 2.0f, 3.0f);
  f32v("(define v []) (loop (x from: 0 upto: 4) (vector/append v x)) v",
       5,
       0.0f,
       1.0f,
       2.0f,
       3.0f,
       4.0f);
  f32v("(define v []) (loop (x from: 0 to: 10 inc: 2) (vector/append v x)) v",
       5,
       0.0f,
       2.0f,
       4.0f,
       6.0f,
       8.0f);
  f32v("(define v []) (loop (x from: 0 upto: 10 inc: 2) (vector/append v x)) v",
       6,
       0.0f,
       2.0f,
       4.0f,
       6.0f,
       8.0f,
       10.0f);
}

void test_vm_col_rgb(void) {
  VM_COMPILE_COL_L("(col/rgb r: 0.1 g: 0.2 b: 0.3 alpha 0.4)", RGB, 0.1f, 0.2f, 0.3f, 0.4f);
  // checking colour_avail
  // TODO: this leaks colour, what should happen instead?
  // VM_COMPILE_F32("(fn (f) (col/rgb r: 0.1 g: 0.2 b: 0.3 alpha 0.4) 5)
  // (f)", 5.0f);
}

void test_vm_math(void) {
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

void test_vm_prng(void) {
  // leaky because the global rng is a vector
  //
  VM_COMPILE_F32_L("(define rng (prng/build seed: 43215 min: 5 max: 20)) "
                   "(prng/value from: rng)",
                   9.1355f);

  // state of rng is changing, returning a different number than previous tests
  VM_COMPILE_F32_L("(define rng (prng/build seed: 43215 min: 5 max: 20)) "
                   "(prng/value from: rng) (prng/value from: rng)",
                   16.5308f);

  // wrapped in a function so that it's not leaky
  VM_COMPILE_F32("(fn (x) (define rng (prng/build seed: 43215 min: 5 max: 20)) "
                 "(prng/value from: rng)) (x)",
                 9.1355f);

  // state of rng is changing, returning a different number than previous tests
  VM_COMPILE_F32("(fn (x) (define rng (prng/build seed: 43215 min: 5 max: 20)) "
                 "(prng/value from: rng) (prng/value from: rng)) (x)",
                 16.5308f);

  // prng/take returning a vector
  VM_COMPILE_F32_L("(define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo "
                   "(prng/values num: 3 from: rng)) (nth n: 0 from: foo)",
                   0.505207f);
  VM_COMPILE_F32_L("(define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo "
                   "(prng/values num: 3 from: rng)) (nth n: 1 from: foo)",
                   -0.406514f);
  VM_COMPILE_F32_L("(define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo "
                   "(prng/values num: 3 from: rng)) (nth n: 2 from: foo)",
                   0.803599f);

  // non-leaky version of above
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo "
                 "(prng/values num: 3 from: rng)) (nth n: 0 from: foo)) (x)",
                 0.505207f);
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo "
                 "(prng/values num: 3 from: rng)) (nth n: 1 from: foo)) (x)",
                 -0.406514f);
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define foo "
                 "(prng/values num: 3 from: rng)) (nth n: 2 from: foo)) (x)",
                 0.803599f);

  // prng, destructuring, multiple args to '+'
  VM_COMPILE_F32("(fn (x) (define rng (prng/build min: -1 max: 1 seed: 3234)) (define [a "
                 "b c] (prng/values num: 3 from: rng)) (+ a b c)) (x)",
                 0.902292f);
}

void test_vm_environmental(void) {
  VM_COMPILE_F32("canvas/width", 1000.0f);
  VM_COMPILE_F32("canvas/height", 1000.0f);
}

void test_vm_interp(void) {
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [0 1] to: [0 100])) "
                 "(interp/value from: i t: 0.5)) (x)",
                 50.0f);

  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [10 20] to: [50 200])) "
                 "(interp/value from: i t: 10.0)) (x)",
                 50.0f);
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [10 20] to: [50 200])) "
                 "(interp/value from: i t: 20.0)) (x)",
                 200.0f);

  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [50 10] to: [100 "
                 "1000])) (interp/value from: i t: 50.0)) (x)",
                 100.0f);
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [50 10] to: [100 "
                 "1000])) (interp/value from: i t: 10.0)) (x)",
                 1000.0f);

  // clamping
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [0 1] to: [0 100] "
                 "clamping: false)) (interp/value from: i t: 2.0)) (x)",
                 200.0f);
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [0 1] to: [0 100] "
                 "clamping: true)) (interp/value from: i t: 2.0)) (x)",
                 100.0f);
  VM_COMPILE_F32("(fn (x) (define i (interp/build from: [0 1] to: [0 100] "
                 "clamping: true)) (interp/value from: i t: -2.0)) (x)",
                 0.0f);
}

void test_vm_function_address(void) {
  VM_COMPILE_F32("(fn (k a: 5) (+ a a)) (fn (l a: 5) (+ a a)) (define foo "
                 "(address-of l)) (fn-call (foo a: 99 b: 88))",
                 198.0f);

  // normal
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo "
                 "(address-of dbl)) (fn-call (foo a: 44))",
                 88.0f);
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo "
                 "(address-of trp)) (fn-call (foo a: 44))",
                 132.0f);

  // invalid arguments - use defaults
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo "
                 "(address-of dbl)) (fn-call (foo z: 44))",
                 10.0f);
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo "
                 "(address-of trp)) (fn-call (foo z: 44))",
                 15.0f);

  // some invalid arguments
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo "
                 "(address-of dbl)) (fn-call (foo z: 100 a: 44))",
                 88.0f);
  VM_COMPILE_F32("(fn (dbl a: 5) (* a 2)) (fn (trp a: 5) (* a 3)) (define foo "
                 "(address-of trp)) (fn-call (foo z: 41 a: 44))",
                 132.0f);
}

void test_vm_repeat(void) {
  // angle is being sent to the function
  f32v("(define v []) (fn (k angle: 0) (vector/append v angle)) (repeat/rotate "
       "fn: (address-of k) copies: 3) v",
       3,
       0.0f,
       120.0f,
       240.0f);

  // num is being sent to the function
  f32v("(define v []) (fn (k copy: 0) (vector/append v copy)) (repeat/rotate "
       "fn: (address-of k) copies: 3) v",
       3,
       0.0f,
       1.0f,
       2.0f);

  // default arguments are being set
  f32v("(define v []) (fn (k angle: 0 shabba: 5.0) (vector/append v (+ shabba "
       "angle))) (repeat/rotate fn: (address-of k) copies: 3) v",
       3,
       5.0f,
       125.0f,
       245.0f);

  f32v("(define v []) (fn (k copy: 0 angle: 0 shabba: 5.0) (vector/append v (+ "
       "copy shabba angle))) (repeat/rotate fn: (address-of k) copies: 3) v",
       3,
       5.0f,
       126.0f,
       247.0f);
}

sen_genotype* genotype_construct(i32 seed_value, char* source) {
  sen_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  sen_env* env = env_allocate();

  sen_node* ast = parser_parse(env->word_lut, source);

  sen_compiler_config compiler_config;
  compiler_config.program_max_size = MAX_TRAIT_PROGRAM_SIZE;
  compiler_config.word_lut         = env->word_lut;

  sen_trait_list* trait_list = trait_list_compile(ast, &compiler_config);

  // using the vm to build the genes
  sen_genotype* genotype = genotype_build_from_program(trait_list, vm, env, seed_value);

  trait_list_return_to_pool(trait_list);
  parser_return_nodes_to_pool(ast);

  env_free(env);
  vm_free(vm);

  return genotype;
}

void unparse_compare(i32 seed_value, char* source, char* expected) {
  sen_systems_startup();

  sen_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  sen_env* env = env_allocate();

  sen_node* ast = parser_parse(env->word_lut, source);

  sen_compiler_config compiler_config;
  compiler_config.program_max_size = MAX_TRAIT_PROGRAM_SIZE;
  compiler_config.word_lut         = env->word_lut;

  sen_trait_list* trait_list = trait_list_compile(ast, &compiler_config);

  // sen_trait *trait = trait_list->traits;
  // program_pretty_print(trait->program);

  // using the vm to build the genes
  sen_genotype* genotype = genotype_build_from_program(trait_list, vm, env, seed_value);

  i32   unparsed_source_size = 1024;
  char* unparsed_source      = (char*)calloc(unparsed_source_size, sizeof(char));

  sen_cursor* cursor = cursor_allocate(unparsed_source, unparsed_source_size);

  unparse(cursor, env->word_lut, ast, genotype);

  if (expected != NULL) {
    TEST_ASSERT_EQUAL_STRING(expected, unparsed_source);
  } else {
    TEST_ASSERT_EQUAL_STRING(source, unparsed_source);
  }

  free(unparsed_source);

  cursor_free(cursor);

  parser_return_nodes_to_pool(ast);
  genotype_return_to_pool(genotype);
  trait_list_return_to_pool(trait_list);

  env_free(env);
  vm_free(vm);

  sen_systems_shutdown();
}

void simplified_unparse_compare(char* source, char* expected) {
  sen_systems_startup();

  sen_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  sen_env* env = env_allocate();

  sen_node* ast = parser_parse(env->word_lut, source);

  sen_compiler_config compiler_config;
  compiler_config.program_max_size = MAX_TRAIT_PROGRAM_SIZE;
  compiler_config.word_lut         = env->word_lut;

  i32   unparsed_source_size = 1024;
  char* unparsed_source      = (char*)calloc(unparsed_source_size, sizeof(char));

  sen_cursor* cursor = cursor_allocate(unparsed_source, unparsed_source_size);

  simplified_unparse(cursor, env->word_lut, ast);

  if (expected != NULL) {
    TEST_ASSERT_EQUAL_STRING(expected, unparsed_source);
  } else {
    TEST_ASSERT_EQUAL_STRING(source, unparsed_source);
  }

  free(unparsed_source);

  cursor_free(cursor);

  parser_return_nodes_to_pool(ast);

  env_free(env);
  vm_free(vm);

  sen_systems_shutdown();
}

void test_genotype(void) {
  sen_genotype* genotype;
  sen_gene*     g;
  sen_var*      v;

  // startup/shutdown here and not in genotype_construct as the tests compare
  // genotypes
  sen_systems_startup();

  {
    genotype = genotype_construct(3421, "(+ 6 {3 (gen/int min: 1 max: 100)})");
    TEST_ASSERT(genotype);
    g = genotype->genes;
    v = g->var;
    assert_sen_var_f32(v, VAR_FLOAT, 81.0f);
    TEST_ASSERT_NULL(g->next); // only 1 gene
    genotype_return_to_pool(genotype);
  }

  {
    genotype = genotype_construct(3421, "(+ 6 {3 (gen/scalar min: 1 max: 100)})");
    TEST_ASSERT(genotype);
    g = genotype->genes;
    v = g->var;
    assert_sen_var_f32(v, VAR_FLOAT, 80.271f);
    TEST_ASSERT_NULL(g->next); // only 1 gene
    genotype_return_to_pool(genotype);
  }

  {
    genotype = genotype_construct(9834, "(+ 6 {3 (gen/int min: 1 max: 100)})");
    TEST_ASSERT(genotype);
    g = genotype->genes;
    v = g->var;
    assert_sen_var_f32(v, VAR_FLOAT, 17.0f);
    TEST_ASSERT_NULL(g->next); // only 1 gene
    genotype_return_to_pool(genotype);
  }

  {
    genotype = genotype_construct(9834, "{(col/rgb r: 0.1) (gen/col alpha: 0.3)}");
    TEST_ASSERT(genotype);
    g = genotype->genes;
    v = g->var;

    TEST_ASSERT_EQUAL(VAR_COLOUR, v->type);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 0.1653f, v->f32_array[0]);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 0.5588f, v->f32_array[1]);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 0.1425f, v->f32_array[2]);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 0.3, v->f32_array[3]);
    TEST_ASSERT_NULL(g->next); // only 1 gene
    genotype_return_to_pool(genotype);
  }

  sen_systems_shutdown();
}

void test_genotype_stray(void) {
  sen_genotype* genotype;
  sen_gene*     g;
  sen_var*      v;

  // startup/shutdown here and not in genotype_construct as the tests compare
  // genotypes
  sen_systems_startup();

  {
    genotype = genotype_construct(3421, "{3 (gen/stray from: 3 by: 0.5)}");
    TEST_ASSERT(genotype);
    g = genotype->genes;
    v = g->var;
    assert_sen_var_f32(v, VAR_FLOAT, 3.300718f);
    TEST_ASSERT_NULL(g->next); // only 1 gene
    genotype_return_to_pool(genotype);
  }

  sen_systems_shutdown();
}

void test_genotype_stray_2d(void) {
  sen_genotype* genotype;
  sen_gene*     g;
  sen_var*      v;

  // startup/shutdown here and not in genotype_construct as the tests compare
  // genotypes
  sen_systems_startup();

  {
    genotype = genotype_construct(3421, "{[100 200] (gen/stray-2d from: [100 200] by: 10)}");
    TEST_ASSERT(genotype);
    g = genotype->genes;
    v = g->var;

    TEST_ASSERT_EQUAL(VAR_2D, v->type);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 106.014359f, v->f32_array[0]);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 207.37352f, v->f32_array[1]);

    TEST_ASSERT_NULL(g->next); // only 1 gene
    genotype_return_to_pool(genotype);
  }

  {
    genotype = genotype_construct(3421, "{[100 200] (gen/stray-2d from: [100 200] by: 90)}");
    genotype = genotype_construct(3423, "{[100 200] (gen/stray-2d from: [100 200] by: 90)}");
    genotype = genotype_construct(3424, "{[100 200] (gen/stray-2d from: [100 200] by: 90)}");
    genotype = genotype_construct(3425, "{[100 200] (gen/stray-2d from: [100 200] by: 90)}");
    genotype = genotype_construct(3427, "{[100 200] (gen/stray-2d from: [100 200] by: 90)}");
    genotype_return_to_pool(genotype);
  }

  sen_systems_shutdown();
}

void test_genotype_vectors(void) {
  sen_genotype* genotype;
  sen_gene*     g;
  sen_var*      v;

  // startup/shutdown here and not in genotype_construct as the tests compare
  // genotypes
  sen_systems_startup();

  {
    genotype = genotype_construct(9834, "{[[0.1 0.2] [0.3 0.4]] (gen/2d)}");
    TEST_ASSERT(genotype);
    g = genotype->genes;
    v = g->var;

    // this will create 2 genes, each one for a VAR_2D

    TEST_ASSERT_EQUAL(VAR_2D, v->type);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 0.1653f, v->f32_array[0]);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 0.5588f, v->f32_array[1]);

    g = g->next;
    v = g->var;
    TEST_ASSERT_EQUAL(VAR_2D, v->type);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 0.1425f, v->f32_array[0]);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 0.0377f, v->f32_array[1]);

    g = g->next;
    TEST_ASSERT_NULL(g);
  }

  {
    genotype = genotype_construct(9834, "{[[0.1 0.2] [0.3 0.4]] (gen/2d min: 50 max: 60)}");
    TEST_ASSERT(genotype);
    g = genotype->genes;
    v = g->var;

    // this will create 2 genes, each one for a VAR_2D

    TEST_ASSERT_EQUAL(VAR_2D, v->type);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 51.6535f, v->f32_array[0]);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 55.5886f, v->f32_array[1]);

    g = g->next;
    v = g->var;
    TEST_ASSERT_EQUAL(VAR_2D, v->type);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 51.4252f, v->f32_array[0]);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 50.3771f, v->f32_array[1]);

    g = g->next;
    TEST_ASSERT_NULL(g);
  }

  sen_systems_shutdown();
}

void test_genotype_multiple_floats(void) {
  sen_genotype* genotype;
  sen_gene*     g;
  sen_var*      v;

  // startup/shutdown here and not in genotype_construct as the tests compare
  // genotypes
  sen_systems_startup();

  {
    genotype = genotype_construct(9834, "{[0.977 0.416 0.171] (gen/scalar)}");
    TEST_ASSERT(genotype);
    g = genotype->genes;
    v = g->var;

    // this will create 3 genes, each one for each float
    TEST_ASSERT_EQUAL(VAR_FLOAT, v->type);

    g = g->next;
    v = g->var;
    TEST_ASSERT_EQUAL(VAR_FLOAT, v->type);

    g = g->next;
    v = g->var;
    TEST_ASSERT_EQUAL(VAR_FLOAT, v->type);

    g = g->next;
    TEST_ASSERT_NULL(g);
  }

  sen_systems_shutdown();
}

sen_genotype* genotype_construct_initial_value(i32 seed_value, char* source) {
  sen_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  sen_env* env = env_allocate();

  sen_node* ast = parser_parse(env->word_lut, source);

  sen_compiler_config compiler_config;
  compiler_config.program_max_size = MAX_TRAIT_PROGRAM_SIZE;
  compiler_config.word_lut         = env->word_lut;

  sen_trait_list* trait_list = trait_list_compile(ast, &compiler_config);

  // using the vm to build the genes
  sen_genotype* genotype = genotype_build_from_program(trait_list, vm, env, seed_value);

  trait_list_return_to_pool(trait_list);
  parser_return_nodes_to_pool(ast);

  env_free(env);
  vm_free(vm);

  return genotype;
}

// applies the given seed to generate a genotype, then compiles a program where
// the expected output is an f32
//
void vm_compile_f32_with_2_genes(char* expr, int seed, f32 expected_res, f32 expected_g1, f32 expected_g2) {
  sen_systems_startup();
  sen_env*     e    = env_allocate();
  sen_genotype* genotype = genotype_construct_initial_value(seed, expr);
  sen_program* prog = sen_compile_program_with_genotype(expr, genotype, e->word_lut, 256);
  // program_pretty_print(prog);
  sen_vm*      vm   = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);

  TEST_ASSERT(genotype);
  sen_gene* g = genotype->genes;
  sen_var* v = g->var;
  assert_sen_var_f32(v, VAR_FLOAT, expected_g1);

  TEST_ASSERT(g->next);
  g = g->next;
  v = g->var;
  assert_sen_var_f32(v, VAR_FLOAT, expected_g2);

  TEST_ASSERT_NULL(g->next); // only 2 genes

  vm_debug_info_reset(vm);
  vm_run(vm, e, prog);

  assert_sen_var_f32(vm_stack_peek(vm), VAR_FLOAT, expected_res);

  vm_free(vm);
  program_free(prog);
  genotype_return_to_pool(genotype);
  env_free(e);
  sen_systems_shutdown();
}

void test_f32_expr_with_genotype(void) {
  // vm_compile_f32_with_seed("(+ 3 4)", 3434, 7.0f);
  // vm_compile_f32_with_seed("(+ 3 {4 (gen/scalar min: 10 max: 20)})", 3434, 17.264217f);

  vm_compile_f32_with_2_genes("(define v {[150 250] (gen/scalar min: 10 max: 99)}) (nth from: v n: 0)",
                              1111, 78.578339f, 78.578339f, 49.573952f);
}

void test_simplified_unparser(void) {
    simplified_unparse_compare("(+ 1 1)", "(+ 1 1)");
    simplified_unparse_compare("(+ 6 {3 (gen/int min: 1 max: 50)})", "(+ 6 3)");
    simplified_unparse_compare("(col/rgb r: {0.4 (gen/scalar)} g: 0.1)", "(col/rgb r: 0.4 g: 0.1)");
    simplified_unparse_compare("{b (gen/select from: '(a b c))}", "b");
    simplified_unparse_compare("{robocop (gen/select from: col/procedural-fn-presets)}", "robocop");
    simplified_unparse_compare("{50 (gen/stray from: 50 by: 20)}", "50");
}

void test_unparser(void) {
  unparse_compare(9875, "(+ 4 2.0)", NULL);
  unparse_compare(9875, "(+ 4 [1 2 3])", NULL);
  unparse_compare(9875, "(+ 4 [1.0 2.22 3.333 4.4444 5.55555])", NULL);
  unparse_compare(9875, "red", NULL);
  unparse_compare(9875, "foo:", NULL);
  unparse_compare(9875, "foo ; some comment \"here\"", NULL);
  unparse_compare(9875, "(fn (a b: 10) (+ b 20))", NULL);
  unparse_compare(
      9875, "(+ 6 {3 (gen/int min: 1 max: 50)})", "(+ 6 {48 (gen/int min: 1 max: 50)})");
  unparse_compare(9875, "(+ 7 { 4 (gen/int min: 2 max: 6)})", "(+ 7 { 6 (gen/int min: 2 max: 6)})");
  unparse_compare(9875, "[8 {3 (gen/int min: 0 max: 9)}]", "[8 {9 (gen/int min: 0 max: 9)}]");

  unparse_compare(6534, "{3.45 (gen/scalar min: 0 max: 9)}", "{7.52 (gen/scalar min: 0 max: 9)}");
  unparse_compare(6534, "{3.4 (gen/scalar min: 0 max: 9)}", "{7.5 (gen/scalar min: 0 max: 9)}");

  unparse_compare(
      6534, "(col/rgb r: {0.4 (gen/scalar)} g: 0.1)", "(col/rgb r: {0.8 (gen/scalar)} g: 0.1)");

  unparse_compare(6534, "{3 (gen/select from: '(4 5 6 7))}", "{7 (gen/select from: '(4 5 6 7))}");

  unparse_compare(5246, "(define c (col/build-procedural preset: robocop alpha: 0.08))", NULL);

  // there was a bug which wasn't correctly traversing the ast to assign genes
  unparse_compare(6542,
                  "(rect position: [500 500] colour: red width: {120 (gen/int min: 80 max: "
                  "400)} height: {140 (gen/int min: 80 max: 670)}) (rect position: [500 "
                  "500] colour: red width: {120 (gen/int min: 80 max: 400)} height: {140 "
                  "(gen/int min: 80 max: 670)}) (rect position: [500 500] colour: red "
                  "width: {120 (gen/int min: 80 max: 400)} height: {140 (gen/int min: 80 "
                  "max: 670)})",
                  "(rect position: [500 500] colour: red width: {91 (gen/int min: 80 max: "
                  "400)} height: {561 (gen/int min: 80 max: 670)}) (rect position: [500 "
                  "500] colour: red width: {228 (gen/int min: 80 max: 400)} height: {257 "
                  "(gen/int min: 80 max: 670)}) (rect position: [500 500] colour: red "
                  "width: {380 (gen/int min: 80 max: 400)} height: {416 (gen/int min: 80 "
                  "max: 670)})");

  unparse_compare(6534, "{b (gen/select from: '(a b c))}", "{c (gen/select from: '(a b c))}");
  unparse_compare(6534,
                  "{red (gen/select from: '(red green blue))}",
                  "{blue (gen/select from: '(red green blue))}");

  unparse_compare(6534,
                  "{rainbow (gen/select from: col/procedural-fn-presets)}",
                  "{robocop (gen/select from: col/procedural-fn-presets)}");

  unparse_compare(9999,
                  "{(col/rgb r: 1 g: 0 b: 0.4 alpha: 1) (gen/col)}",
                  "{(col/rgb r: 0.00 g: 0.72 b: 0.16 alpha: 0.26) (gen/col)}");
  unparse_compare(9999,
                  "{(col/rgb r: 1 g: 0 b: 0.4 alpha: 1) (gen/col alpha: 1)}",
                  "{(col/rgb r: 0.00 g: 0.72 b: 0.16 alpha: 1.00) (gen/col alpha: 1)}");

  unparse_compare(6534, "{50 (gen/stray from: 50 by: 20)}", "{63 (gen/stray from: 50 by: 20)}");
}

void test_unparser_vectors(void) {
  unparse_compare(
      9999, "{[[1.00 2.00] [3.00 4.00]] (gen/2d)}", "{[[0.00 0.72] [0.16 0.26]] (gen/2d)}");

  unparse_compare(9999,
                  "{[[  1.00   2.00  ] [  3.00   4.00  ]] (gen/2d)}",
                  "{[[  0.00   0.72  ] [  0.16   0.26  ]] (gen/2d)}");

  unparse_compare(9999,
                  "{[[10 20] [30 40]] (gen/2d min: 60 max: 70)}",
                  "{[[60 67] [62 63]] (gen/2d min: 60 max: 70)}");

  unparse_compare(9999,
                  "{[[ 50.1 60.23 ] [ 70.456 80.7890 ]] (gen/2d min: 40 max: 90)}",
                  "{[[ 40.1 76.16 ] [ 47.912 52.8556 ]] (gen/2d min: 40 max: 90)}");

  unparse_compare(9999,
                  "{ [ [ 50.1 60.23 ] [ 70.456 80.7890 ]] (gen/2d min: 40 max: 90) }",
                  "{ [ [ 40.1 76.16 ] [ 47.912 52.8556 ]] (gen/2d min: 40 max: 90) }");
}

void test_unparser_single_trait_vectors(void) {
  unparse_compare(1298,
                  "{[10 20] (gen/stray-2d from: [10 20] by: 5)}",
                  "{[13 19] (gen/stray-2d from: [10 20] by: 5)}");
}

void test_unparser_multiple_floats(void) {
  unparse_compare(3455, "{[0.977 0.416 0.171] (gen/scalar)}", "{[0.022 0.737 0.898] (gen/scalar)}");
}

// serialize/deserialize sen_var

void compare_sen_var(sen_var* a, sen_var* b) {
  TEST_ASSERT_EQUAL(a->type, b->type);
  switch (a->type) {
  case VAR_INT:
    TEST_ASSERT_EQUAL(a->value.i, b->value.i);
    break;
  case VAR_FLOAT:
    TEST_ASSERT_EQUAL_FLOAT(a->value.f, b->value.f);
    break;
  case VAR_BOOLEAN:
    TEST_ASSERT_EQUAL(a->value.i, b->value.i);
    break;
  case VAR_LONG:
    TEST_ASSERT_EQUAL(a->value.l, b->value.l);
    break;
  case VAR_NAME:
    TEST_ASSERT_EQUAL(a->value.i, b->value.i);
    break;
  case VAR_VECTOR:
    TEST_ASSERT_EQUAL(a->value.v, b->value.v);
    break;
  case VAR_COLOUR:
    TEST_ASSERT_EQUAL(a->value.i, b->value.i);
    TEST_ASSERT_EQUAL(a->f32_array[0], b->f32_array[0]);
    TEST_ASSERT_EQUAL(a->f32_array[1], b->f32_array[1]);
    TEST_ASSERT_EQUAL(a->f32_array[2], b->f32_array[2]);
    TEST_ASSERT_EQUAL(a->f32_array[3], b->f32_array[3]);
    break;
  case VAR_2D:
    TEST_ASSERT_EQUAL(a->f32_array[0], b->f32_array[0]);
    TEST_ASSERT_EQUAL(a->f32_array[1], b->f32_array[1]);
    break;
  default:
    SEN_ERROR("unknown sen_var type");
  }
}

void compare_sen_bytecode(sen_bytecode* a, sen_bytecode* b) {
  TEST_ASSERT_EQUAL(a->op, b->op);
  compare_sen_var(&(a->arg0), &(b->arg0));
  compare_sen_var(&(a->arg1), &(b->arg1));
}

void compare_sen_program(sen_program* a, sen_program* b) {
  TEST_ASSERT_EQUAL(a->code_size, b->code_size);
  for (i32 i = 0; i < a->code_size; i++) {
    compare_sen_bytecode(&(a->code[i]), &(b->code[i]));
  }
}

void serialize_deserialize_var(sen_var* var) {
  i32   buffer_size = 128;
  char* buffer      = (char*)calloc(buffer_size, sizeof(char));

  sen_var out;
  bool    res;

  sen_cursor* cursor = cursor_allocate(buffer, buffer_size);

  res = var_serialize(cursor, var);

  TEST_ASSERT_TRUE(res);

  cursor_reset(cursor);

  res = var_deserialize(&out, cursor);

  TEST_ASSERT_TRUE(res);

  compare_sen_var(var, &out);

  cursor_free(cursor);
  free(buffer);
}

void test_serialization(void) {
  sen_var var;

  {
    var.type    = VAR_INT;
    var.value.i = 42;
    serialize_deserialize_var(&var);
  }
  {
    var.type    = VAR_FLOAT;
    var.value.f = 12.34f;
    serialize_deserialize_var(&var);
  }
  {
    var.type    = VAR_BOOLEAN;
    var.value.i = 0;
    serialize_deserialize_var(&var);
    var.value.i = 1;
    serialize_deserialize_var(&var);
  }
  {
    var.type    = VAR_LONG;
    var.value.l = (u64)934872325L;
    serialize_deserialize_var(&var);
  }
  {
    var.type    = VAR_NAME;
    var.value.i = 12;
    serialize_deserialize_var(&var);
  }
  {
    var.type         = VAR_COLOUR;
    var.value.i      = RGB;
    var.f32_array[0] = 0.1f;
    var.f32_array[1] = 0.2f;
    var.f32_array[2] = 0.3f;
    var.f32_array[3] = 0.4f;
    serialize_deserialize_var(&var);
  }
  {
    var.type         = VAR_2D;
    var.f32_array[0] = 0.5f;
    var.f32_array[1] = 0.6f;
    serialize_deserialize_var(&var);
  }

  i32   buffer_size = 128;
  char* buffer      = (char*)calloc(buffer_size, sizeof(char));

  sen_cursor* cursor = cursor_allocate(buffer, buffer_size);

  {
    // double serialize
    var.type    = VAR_INT;
    var.value.i = 12;

    cursor_clear(cursor);
    var_serialize(cursor, &var);
    cursor_sprintf(cursor, " ");

    sen_var var2;
    var2.type    = VAR_INT;
    var2.value.i = 34;
    var_serialize(cursor, &var2);

    cursor_reset(cursor);

    sen_var out_var;
    var_deserialize(&out_var, cursor);
    compare_sen_var(&out_var, &var);
    cursor_eat_space(cursor);
    var_deserialize(&out_var, cursor);
    compare_sen_var(&out_var, &var2);
  }

  {
    // sen_bytecode
    sen_bytecode bytecode;
    bytecode.op           = STORE;
    bytecode.arg0.type    = VAR_INT;
    bytecode.arg0.value.i = 2;
    bytecode.arg1.type    = VAR_INT;
    bytecode.arg1.value.i = 4;

    cursor_clear(cursor);
    bytecode_serialize(cursor, &bytecode);

    cursor_reset(cursor);

    sen_bytecode out_bytecode;
    bytecode_deserialize(&out_bytecode, cursor);

    compare_sen_bytecode(&out_bytecode, &bytecode);
  }

  cursor_free(cursor);
  free(buffer);
}

void test_serialization_program(void) {
  parser_subsystem_startup();

  sen_env*     env     = env_allocate();
  sen_program* program = sen_compile_program("(gen/int min: 2 max: 6)", env->word_lut, 256);

  i32   buffer_size = 4096;
  char* buffer      = (char*)calloc(buffer_size, sizeof(char));

  sen_cursor* cursor = cursor_allocate(buffer, buffer_size);

  bool res = program_serialize(cursor, program);
  TEST_ASSERT_TRUE(res);

  cursor_reset(cursor);

  sen_program* out = program_allocate(256);
  res              = program_deserialize(out, cursor);
  TEST_ASSERT_TRUE(res);

  compare_sen_program(out, program);

  free(buffer);
  cursor_free(cursor);
  program_free(program);
  program_free(out);
  env_free(env);

  parser_subsystem_shutdown();
}

void test_serialization_genotype(void) {
  sen_genotype* genotype;
  sen_gene*     g;

  i32         buffer_size = 4096;
  char*       buffer      = (char*)calloc(buffer_size, sizeof(char));
  sen_cursor* cursor      = cursor_allocate(buffer, buffer_size);
  bool        res;

  sen_systems_startup();

  {
    cursor_reset(cursor);

    genotype = genotype_construct(3421, "(+ 6 {3 (gen/int min: 1 max: 100)})");
    TEST_ASSERT(genotype);

    res = genotype_serialize(cursor, genotype);
    TEST_ASSERT_TRUE(res);

    cursor_reset(cursor);

    sen_genotype* out = genotype_get_from_pool();
    res               = genotype_deserialize(out, cursor);
    TEST_ASSERT_TRUE(res);

    g = out->genes;
    assert_sen_var_f32(g->var, VAR_FLOAT, 81.0f);

    genotype_return_to_pool(out);
    genotype_return_to_pool(genotype);
  }

  {
    cursor_reset(cursor);

    genotype = genotype_construct(7534, "(+ {2 (gen/int min: 1 max: 30)} {5 (gen/int min: 1 max: 30)})");
    TEST_ASSERT(genotype);

    res = genotype_serialize(cursor, genotype);
    TEST_ASSERT_TRUE(res);

    cursor_reset(cursor);

    sen_genotype* out = genotype_get_from_pool();
    res               = genotype_deserialize(out, cursor);
    TEST_ASSERT_TRUE(res);

    g = out->genes;
    assert_sen_var_f32(g->var, VAR_FLOAT, 5.0f);

    g = g->next;
    assert_sen_var_f32(g->var, VAR_FLOAT, 4.0f);

    g = g->next;
    TEST_ASSERT_NULL(g);

    genotype_return_to_pool(out);
    genotype_return_to_pool(genotype);
  }

  sen_systems_shutdown();

  cursor_free(cursor);
  free(buffer);
}

void test_serialization_genotype_list(void) {
  sen_systems_startup();

  sen_genotype_list* genotype_list;
  sen_genotype*      genotype;
  sen_gene*          g;

  genotype_list = genotype_list_get_from_pool();

  i32         buffer_size = 4096;
  char*       buffer      = (char*)calloc(buffer_size, sizeof(char));
  sen_cursor* cursor      = cursor_allocate(buffer, buffer_size);
  bool        res;

  {
    cursor_reset(cursor);

    genotype = genotype_construct(7534, "(+ {2 (gen/int min: 1 max: 30)} {5 (gen/int min: 1 max: 30)})");
    TEST_ASSERT(genotype);
    genotype_list_add_genotype(genotype_list, genotype);

    genotype = genotype_construct(2313, "(+ {2 (gen/int min: 1 max: 30)} {5 (gen/int min: 1 max: 30)})");
    TEST_ASSERT(genotype);
    genotype_list_add_genotype(genotype_list, genotype);

    genotype = genotype_construct(4562, "(+ {2 (gen/int min: 1 max: 30)} {5 (gen/int min: 1 max: 30)})");
    TEST_ASSERT(genotype);
    genotype_list_add_genotype(genotype_list, genotype);

    res = genotype_list_serialize(cursor, genotype_list);
    TEST_ASSERT_TRUE(res);

    cursor_reset(cursor);

    sen_genotype_list* out = genotype_list_get_from_pool();
    res                    = genotype_list_deserialize(out, cursor);
    TEST_ASSERT_TRUE(res);

    i32 count = genotype_list_count(out);
    TEST_ASSERT_EQUAL(3, count);

    genotype = out->genotypes;
    g        = genotype->genes;
    assert_sen_var_f32(g->var, VAR_FLOAT, 5.0f);
    g = g->next;
    assert_sen_var_f32(g->var, VAR_FLOAT, 4.0f);
    g = g->next;
    TEST_ASSERT_NULL(g);

    genotype = genotype->next;
    g        = genotype->genes;
    assert_sen_var_f32(g->var, VAR_FLOAT, 4.0f);
    g = g->next;
    assert_sen_var_f32(g->var, VAR_FLOAT, 5.0f);
    g = g->next;
    TEST_ASSERT_NULL(g);

    genotype = genotype->next;
    g        = genotype->genes;
    assert_sen_var_f32(g->var, VAR_FLOAT, 23.0f);
    g = g->next;
    assert_sen_var_f32(g->var, VAR_FLOAT, 18.0f);
    g = g->next;
    TEST_ASSERT_NULL(g);

    genotype = genotype->next;
    TEST_ASSERT_NULL(genotype);

    genotype_list_return_to_pool(out);
  }

  cursor_free(cursor);
  free(buffer);

  genotype_list_return_to_pool(genotype_list);

  sen_systems_shutdown();
}

void test_serialization_trait_list(void) {
  char* source = "(rect position: [500 500] width: {100 (gen/int min: 20 max: "
                 "200)} height: {30 (gen/int min: 10 max: 40)})";

  sen_systems_startup();

  sen_vm*  vm  = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  sen_env* env = env_allocate();

  sen_node* ast = parser_parse(env->word_lut, source);

  sen_compiler_config compiler_config;
  compiler_config.program_max_size = MAX_TRAIT_PROGRAM_SIZE;
  compiler_config.word_lut         = env->word_lut;

  sen_trait_list* trait_list = trait_list_compile(ast, &compiler_config);

  i32         buffer_size = 4096;
  char*       buffer      = (char*)calloc(buffer_size, sizeof(char));
  sen_cursor* cursor      = cursor_allocate(buffer, buffer_size);
  bool        res         = trait_list_serialize(cursor, trait_list);
  TEST_ASSERT_TRUE(res);

  cursor_reset(cursor);

  sen_trait_list* out = trait_list_get_from_pool();
  res                 = trait_list_deserialize(out, cursor);
  TEST_ASSERT_TRUE(res);

  i32 count = trait_list_count(out);
  TEST_ASSERT_EQUAL(2, count);

  compare_sen_var(out->traits->initial_value, trait_list->traits->initial_value);
  compare_sen_program(out->traits->program, trait_list->traits->program);

  compare_sen_var(out->traits->next->initial_value, trait_list->traits->next->initial_value);
  compare_sen_program(out->traits->next->program, trait_list->traits->next->program);

  parser_return_nodes_to_pool(ast);

  cursor_free(cursor);
  free(buffer);
  trait_list_return_to_pool(trait_list);
  trait_list_return_to_pool(out);

  env_free(env);
  vm_free(vm);

  sen_systems_shutdown();
}

void assert_rgb_hsluv(f32 r, f32 g, f32 b, f32 h, f32 s, f32 l) {
  sen_colour rgb;
  rgb.format     = RGB;
  rgb.element[0] = r;
  rgb.element[1] = g;
  rgb.element[2] = b;
  rgb.element[3] = 1.0f;

  sen_colour hsluv;
  hsluv.format     = HSLuv;
  hsluv.element[0] = h;
  hsluv.element[1] = s;
  hsluv.element[2] = l;
  hsluv.element[3] = 1.0f;

  sen_colour res;
  colour_clone_as(&res, &rgb, HSLuv);

  TEST_ASSERT_EQUAL(HSLuv, res.format);
  TEST_ASSERT_FLOAT_WITHIN(0.001f, h, res.element[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.001f, s, res.element[1]);
  TEST_ASSERT_FLOAT_WITHIN(0.001f, l, res.element[2]);
  TEST_ASSERT_EQUAL_FLOAT(1.0f, res.element[3]);

  colour_clone_as(&res, &hsluv, RGB);

  TEST_ASSERT_EQUAL(RGB, res.format);
  TEST_ASSERT_FLOAT_WITHIN(0.001f, r, res.element[0]);
  TEST_ASSERT_FLOAT_WITHIN(0.001f, g, res.element[1]);
  TEST_ASSERT_FLOAT_WITHIN(0.001f, b, res.element[2]);
  TEST_ASSERT_EQUAL_FLOAT(1.0f, res.element[3]);
}

void test_rgb_hsluv_conversion(void) {
  FILE*  fp = fopen("colours_rgb_hsluv.txt", "r");
  double r, g, b, h, s, l;
  int    read;
  i32    count = 0;

  read = fscanf(fp, "%lf %lf %lf %lf %lf %lf", &r, &g, &b, &h, &s, &l);
  while (read == 6) {
    count++;
    assert_rgb_hsluv((f32)r, (f32)g, (f32)b, (f32)h, (f32)s, (f32)l);
    read = fscanf(fp, "%lf %lf %lf %lf %lf %lf", &r, &g, &b, &h, &s, &l);
  }

  TEST_ASSERT_EQUAL(4096, count);

  fclose(fp);
}

void bug_f32_expr_with_genotype(void) {
  // vm_compile_f32_with_seed("(+ 3 4)", 3434, 7.0f);
  // vm_compile_f32_with_seed("(+ 3 {4 (gen/scalar min: 10 max: 20)})", 3434, 17.264217f);

  vm_compile_f32_with_2_genes("(define v {[364.374 334.649] (gen/stray-2d from: [100 200] by: 2000)}) (nth from: v n: 0)",
                              1111, 78.578339f, 78.578339f, 49.573952f);
}

int main(void) {
  // timing();

  if (INAME_NUMBER_OF_KNOWN_WORDS >= NATIVE_START) {
    SEN_LOG("WARNING: keywords are overwriting into NATIVE_START area");
  }

  UNITY_BEGIN();

  // RUN_TEST(debug_lang_interpret_mem); // for debugging/development
  // RUN_TEST(test_prng);
  // todo: test READ_STACK_ARG_COORD4

#if 0
  RUN_TEST(test_macro_pool);
  RUN_TEST(test_mathutil);
  RUN_TEST(test_parser);
  RUN_TEST(test_uv_mapper);
  RUN_TEST(test_colour);
  RUN_TEST(test_strtof);

  RUN_TEST(test_vm_bugs);
  RUN_TEST(test_vm_bytecode);
  RUN_TEST(test_vm_callret);
  RUN_TEST(test_vm_native);
  RUN_TEST(test_vm_destructure);
  RUN_TEST(test_vm_2d);
  RUN_TEST(test_vm_vector);
  RUN_TEST(test_vm_vector_append);
  RUN_TEST(test_vm_fence);
  RUN_TEST(test_vm_loop);
  RUN_TEST(test_vm_col_rgb);
  RUN_TEST(test_vm_math);
  RUN_TEST(test_vm_prng);
  RUN_TEST(test_vm_environmental);
  RUN_TEST(test_vm_interp);
  RUN_TEST(test_vm_function_address);
  RUN_TEST(test_vm_repeat);

  RUN_TEST(test_genotype);
  RUN_TEST(test_genotype_stray);
  RUN_TEST(test_genotype_stray_2d);
  RUN_TEST(test_genotype_vectors);
  RUN_TEST(test_genotype_multiple_floats);

  RUN_TEST(test_f32_expr_with_genotype);

  RUN_TEST(test_simplified_unparser);
  RUN_TEST(test_unparser);
  RUN_TEST(test_unparser_vectors);
  RUN_TEST(test_unparser_single_trait_vectors);
  RUN_TEST(test_unparser_multiple_floats);

  RUN_TEST(test_serialization);
  RUN_TEST(test_serialization_program);
  RUN_TEST(test_serialization_genotype);
  RUN_TEST(test_serialization_genotype_list);
  RUN_TEST(test_serialization_trait_list);

  RUN_TEST(test_rgb_hsluv_conversion);

#else

  RUN_TEST(bug_f32_expr_with_genotype);

#endif

  return UNITY_END();
}
