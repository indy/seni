#include "seni_lang.h"
#include "seni_config.h"
#include "seni_matrix.h"
#include "seni_mathutil.h"

#include "time.h"
#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include <stdio.h>              /* for debug only */

#include "utlist.h"

// global keyword variables
#define KEYWORD(val,_,name) i32 g_keyword_iname_##name = KEYWORD_START + val;
#include "seni_keywords.h"
#undef KEYWORD


#ifdef SENI_DEBUG_MODE

void vm_debug_info_reset(seni_vm *vm)
{
  slab_reset(&(vm->heap_slab_info));
  vm->opcodes_executed = 0;
}

void vm_debug_info_print(seni_vm *vm)
{
  printf("*** vm_debug_info_print ***\n");
  slab_print(&(vm->heap_slab_info), "heap slab");
  printf("bytecodes executed:\t%llu\n", (long long unsigned int)(vm->opcodes_executed));
  printf("bytecode execution time:\t%d msec\n", vm->execution_time);
}

#endif


// for parsing
seni_node *consume_item();

void string_copy_len(char **dst, char *src, size_t len)
{
  char *c = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(c, src, len);
  c[len] = '\0';

  *dst = c;
}

/* returns 0 if not found */
i32 lookup_name(char **words, i32 word_count, i32 offset, char *string, size_t len)
{
  i32 i = 0;
  for (i = 0; i < word_count; i++) {
    char *name = words[i];
    bool found = true;
    /* can't use string_compare since 'string' could be a substring */
    size_t j = 0;
    for (j = 0; j < len; j++) {
      if (name[j] == '\0' || (name[j] != string[j])) {
        found = false;
        break;
      }
    }
    /* searched all of 'string' and the early exit wasn't triggered */
    if (name[j] == '\0' && found) {
      return i + offset;
    }
  }

  return -1;  
}

i32 wlut_lookup_(seni_word_lut *wlut, char *string, size_t len)
{
  i32 native = lookup_name(wlut->native, wlut->native_count, NATIVE_START, string, len);
  if (native != -1) {
    return native;
  }

  i32 keyword = lookup_name(wlut->keyword, wlut->keyword_count, KEYWORD_START, string, len);
  if (keyword != -1) {
    return keyword;
  }

  i32 word = lookup_name(wlut->word, wlut->word_count, WORD_START, string, len);
  if (word != -1) {
    return word;
  }

  return -1;
}

i32 wlut_lookup_or_add(seni_word_lut *wlut, char *string, size_t len)
{
  i32 iname = wlut_lookup_(wlut, string, len);
  if (iname != -1) {
    return iname;
  }

  /* string is not in the table and there's no room for another entry */
  if (wlut->word_count >= MAX_WORD_LOOKUPS) {
    return -1;
  }

  // the string is not in the lookup table, so add it
  string_copy_len(&(wlut->word[wlut->word_count]), string, len);
  wlut->word_count++;

  return wlut->word_count - 1;
}

void wlut_free_keywords(seni_word_lut *wlut)
{
  for( int i = 0; i < MAX_KEYWORD_LOOKUPS; i++) {
    if (wlut->keyword[i]) {
      free(wlut->keyword[i]);
    }
    wlut->keyword[i] = 0;      
  }
  wlut->keyword_count = 0;
}

void wlut_free_natives(seni_word_lut *wlut)
{
  for( int i = 0; i < MAX_NATIVE_LOOKUPS; i++) {
    if (wlut->native[i]) {
      free(wlut->native[i]);
    }
    wlut->native[i] = 0;      
  }
  wlut->native_count = 0;
}

void wlut_free_words(seni_word_lut *wlut)
{
  for( int i = 0; i < MAX_WORD_LOOKUPS; i++) {
    if (wlut->word[i]) {
      free(wlut->word[i]);
    }
    wlut->word[i] = 0;      
  }
  wlut->word_count = 0;
}

seni_word_lut *wlut_allocate()
{
  seni_word_lut *wl = (seni_word_lut *)calloc(1, sizeof(seni_word_lut));
  return wl;
}

void wlut_free(seni_word_lut *wlut)
{
  wlut_free_words(wlut);
  wlut_free_keywords(wlut);
  wlut_free_natives(wlut);
  free(wlut);
}

bool is_minus(char c)
{
  return c == '-';
}

bool is_period(char c)
{
  return c == '.';
}

bool is_whitespace(char c)
{
  return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == ',';
}

bool is_digit(char c)
{
  return c >= '0' && c <= '9';
}

bool is_alpha(char c)
{
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}

bool is_symbol(char c)
{
  return c == '+' || c == '-' || c == '*' || c == '/' || c == '=' ||
    c == '!' || c == '@' || c == '#' || c == '$' || c == '%' ||
    c == '^' || c == '&' || c == '<' || c == '>' || c == '?';
}

bool is_list_start(char c)
{
  return c == '(';
}

bool is_list_end(char c)
{
  return c == ')';
}

bool is_vector_start(char c)
{
  return c == '[';
}

bool is_vector_end(char c)
{
  return c == ']';
}

bool is_alterable_start(char c)
{
  return c == '{';
}

bool is_alterable_end(char c)
{
  return c == '}';
}

bool is_quoted_string(char c)
{
  return c == '"';
}

bool is_quote_abbreviation(char c)
{
  return c == '\'';
}

bool is_comment(char c)
{
  return c == ';';
}

bool is_newline(char c)
{
  return c == '\n';
}

bool is_label(char *s, size_t word_len)
{
  return s[word_len] == ':';
}

bool has_period(char *s)
{
  size_t i = 0;
  char c = s[i];

  while (c != 0) {
    if (is_period(c)) {
      return true;
    }
    if (is_whitespace(c)) {
      return false;
    }
    i++;
    c = s[i];
  }

  return false;
}

char *find_next(char *s, char target)
{
  while (*s != 0){
    if (*s == target) {
      return s;
    }
    s++;
  }
  return NULL;
}

seni_node *build_text_lookup_node_from_string(seni_word_lut *wlut, seni_node_type type, char *string)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  size_t len = strlen(string);

  i32 k = wlut_lookup_or_add(wlut, string, len);
  if (k == -1) {
    return NULL;
  }

  node->type = type;
  node->value.i = k;
  
  return node;
}

seni_node *build_text_lookup_node_of_length(seni_word_lut *wlut, char **src, seni_node_type type, size_t len)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));

  i32 k = wlut_lookup_or_add(wlut, *src, len);
  if (k == -1) {
    return NULL;
  }

  node->type = type;
  node->value.i = k;
  *src += len;

  return node;
}

// allocate memory for comments and whitespace rather than using the lookup table
//
seni_node *build_text_node_of_length(char **src, seni_node_type type, size_t len)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = type;

  char *str = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(str, *src, len);
  str[len] = '\0';

  *src += len;
  
  node->value.s = str;
  
  return node;
}

seni_node *consume_list(seni_word_lut *wlut, char **src)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_LIST;

  (*src)++; // (

  while (1) {
    if (is_list_end(**src)) {
      (*src)++; // )
      return node;
    }

    seni_node *child = consume_item(wlut, src);
    if (child == NULL) {
      SENI_ERROR("unable to consume element of list");
      return NULL;
    }

    DL_APPEND(node->value.first_child, child);
  }
}

seni_node *consume_vector(seni_word_lut *wlut, char **src)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_VECTOR;

  (*src)++; // [
  
  while (1) {
    if (is_vector_end(**src)) {
      (*src)++; // ]
      return node;
    }

    seni_node *child = consume_item(wlut, src);
    if (child == NULL) {
      SENI_ERROR("unable to consume element of vector");
      return NULL;
    }

    DL_APPEND(node->value.first_child, child);
  }
}

seni_node *consume_alterable(seni_word_lut *wlut, char **src)
{
  seni_node *node;
  seni_node *parameter_prefix = NULL;
  seni_node *c;
  
  (*src)++; // {
  
  while (1) {
    c = consume_item(wlut, src);
    if (c == NULL) {
      SENI_ERROR("unable to consume element of alterable");
      return NULL;
    }

    if (c->type == NODE_COMMENT || c->type == NODE_WHITESPACE) {
      DL_APPEND(parameter_prefix, c);
    } else {
      node = c;
      node->alterable = true;
      node->parameter_prefix = parameter_prefix;
      break;
    }
  }

  if (node->type != NODE_INT
      && node->type != NODE_FLOAT
      && node->type != NODE_NAME
      && node->type != NODE_LIST
      && node->type != NODE_VECTOR)
    {
      SENI_ERROR("non-mutable node within curly brackets: %s", node_type_name(node));
      return NULL;
    }
  
  while (1) {
    if (is_alterable_end(**src)) {
      (*src)++; // }
      return node;
    }

    seni_node *child = consume_item(wlut, src);
    if (child == NULL) {
      SENI_ERROR("unable to consume element of bracket");
      return NULL;
    }

    DL_APPEND(node->parameter_ast, child);
  }
}

seni_node *consume_quoted_form(seni_word_lut *wlut, char **src)
{
  (*src)++; // '
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_LIST;

  seni_node *quote_name = build_text_lookup_node_from_string(wlut, NODE_NAME, "quote");
  DL_APPEND(node->value.first_child, quote_name);

  char *wst = " ";
  seni_node *ws = build_text_node_of_length(&wst, NODE_WHITESPACE, 1);
  DL_APPEND(node->value.first_child, ws);

  seni_node *child = consume_item(wlut, src);
  DL_APPEND(node->value.first_child, child);

  return node;
}
/*
seni_node *consume_int(char **src)
{
  char *end_ptr;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_INT;
  node->value.i = (i32)strtoimax(*src, &end_ptr, 10);

  *src = end_ptr;
  
  return node;
}
*/
seni_node *consume_float(char **src)
{
  char *end_ptr;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_FLOAT;
  node->value.f = (f32)strtof(*src, &end_ptr);

  *src = end_ptr;
  
  return node;
}

seni_node *consume_name(seni_word_lut *wlut, char **src)
{
  size_t i = 0;
  char *rem = *src;

  while(rem[i]) {
    char c = rem[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  seni_node *node = build_text_lookup_node_of_length(wlut, src, NODE_NAME, i);

  return node;
}

seni_node *consume_string(seni_word_lut *wlut, char **src)
{
  (*src)++; // skip the first \"

  char *next_quote = find_next(*src, '\"');
  if (next_quote == NULL) {
    return NULL;
  }

  size_t string_len = next_quote - *src;

  seni_node *node = build_text_lookup_node_of_length(wlut, src, NODE_STRING, string_len);

  (*src)++; // skip the second \"
  
  return node;
}

seni_node *consume_label(seni_word_lut *wlut, char **src)
{
  size_t i = 0;
  char *rem = *src;

  while(rem[i]) {
    char c = rem[i];
    if (!is_alpha(c) && !is_digit(c) && !is_symbol(c)) {
      break;
    }
    i++;
  }

  // read the label name - the ':' character
  seni_node *node = build_text_lookup_node_of_length(wlut, src, NODE_LABEL, i);

  if (**src != ':') {
    return NULL;
  }

  (*src)++;        /* the remaining should skip past the ':' */

  return node;
}

seni_node *consume_comment(char **src)
{
  size_t i = 0;
  char *rem = *src;
  
  while (rem[i]) {
    char c = rem[i];
    if (is_newline(c)) {
      break;
    }
    i++;
  }

  seni_node *node = build_text_node_of_length(src, NODE_COMMENT, i);

  if (is_newline(*rem)) {
    (*src)++;        /* skip past the newline */
  }
    
  return node;
}

seni_node *consume_whitespace(char **src)
{
  size_t i = 0;
  char *rem = *src;
  char c = rem[i];
  
  while(c) {
    if (!is_whitespace(c)) {
      break;
    }
    i++;
    c = rem[i];
  }

  seni_node *node = build_text_node_of_length(src, NODE_WHITESPACE, i);

  return node;
}

seni_node *consume_item(seni_word_lut *wlut, char **src)
{
  char c = **src;

  if (is_whitespace(c)) {
    return consume_whitespace(src);
  }

  if (is_quote_abbreviation(c)) {
    return consume_quoted_form(wlut, src);
  }

  if (is_list_start(c)) {
    return consume_list(wlut, src);
  }

  if (is_list_end(c)) {
    return NULL;                /* 'mismatched closing parens' */
  }

  if (is_vector_start(c)) {
    return consume_vector(wlut, src);
  }

  if (is_vector_end(c)) {
    return NULL;                /* 'mismatched closing square brackets' */
  }

  if (is_alterable_start(c)) {
    return consume_alterable(wlut, src);
  }

  if (is_alterable_end(c)) {
    return NULL;                /* 'mismatched closing alterable brackets' */
  }

  if (is_quoted_string(c)) {
    return consume_string(wlut, src);
  }

  if (is_alpha(c) || is_minus(c) || is_symbol(c)) {
    // doesn't begin with -[0..9]
    if (!(is_minus(c) && *(*src + 1) != 0 && is_digit(*(*src + 1)))) {

      char *s = *src;
      size_t word_len = 0;

      while(*s != 0) {
        if (!is_alpha(*s) && !is_digit(*s) && !is_symbol(*s)) {
          break;
        }
        word_len++;
        s++;
      }
      
      if (is_label(*src, word_len)) {
        return consume_label(wlut, src);
      }

      return consume_name(wlut, src);
    }
  }
  
  if (is_digit(c) || is_minus(c) || is_period(c)) {
    return consume_float(src);
  }

  if (is_comment(c)) {
    return consume_comment(src);
  }
  return NULL;
}

char *node_type_name(seni_node *node)
{
  switch(node->type) {
  case NODE_LIST:       return "NODE_LIST";
  case NODE_VECTOR:     return "NODE_VECTOR";
  case NODE_INT:        return "NODE_INT";
  case NODE_FLOAT:      return "NODE_FLOAT";
  case NODE_NAME:       return "NODE_NAME";
  case NODE_LABEL:      return "NODE_LABEL";
  case NODE_STRING:     return "NODE_STRING";
  case NODE_WHITESPACE: return "NODE_WHITESPACE";
  case NODE_COMMENT:    return "NODE_COMMENT";
  default: return "unknown seni_node type";
  };
}

void parser_free_nodes(seni_node *nodes)
{
  seni_node *node = nodes;
  seni_node *next;

  while(node != NULL) {
    if (node->type == NODE_LIST && node->value.first_child) {
      parser_free_nodes(node->value.first_child);
    }
    if (node->parameter_ast) {
      parser_free_nodes(node->parameter_ast);
    }
    if (node->parameter_prefix) {
      parser_free_nodes(node->parameter_prefix);
    }
    
    next = node->next;

    if (node->type == NODE_COMMENT || node->type == NODE_WHITESPACE) {
      // freeing a pointer in a union, so make sure that the value in
      // the union only comes from the 's' component and not 'i' or 'f'
      //
      if (node->value.s != NULL) {
        free(node->value.s);
      }
    }

    // printf("freeing node: %s %u\n", node_type_name(node), (u32)node);
    free(node);
    
    node = next;
  }
}

seni_node *parser_parse(seni_word_lut *wlut, char *s)
{
  if (s == NULL) {
    return NULL;
  }

  char **src = &s;

  seni_node *nodes = NULL;
  seni_node *node;

  while(**src) {
    node = consume_item(wlut, src);
    if (node == NULL) {
      // clean up and fuck off
      parser_free_nodes(nodes);
      return NULL;
    }

    DL_APPEND(nodes, node);
  }

  // NOTE: not strictly a tree as the ast root could have siblings
  return nodes;
}

seni_value_in_use get_value_in_use(seni_var_type type)
{
  switch(type) {
  case VAR_FLOAT:
    return USE_F;
  case VAR_LONG:
    return USE_L;
  case VAR_VEC_HEAD:
    return USE_V;
  case VAR_COLOUR:
    return USE_I;
  case VAR_VEC_RC:
    return USE_I;
  default:
    // default to something even though VAR_2D etc aren't going to use anything in the value union
    return USE_I;
  };
}

char *var_type_name(seni_var *var)
{
  switch(var->type) {
  case VAR_INT:      return "VAR_INT";
  case VAR_FLOAT:    return "VAR_FLOAT";
  case VAR_BOOLEAN:  return "VAR_BOOLEAN";
  case VAR_LONG:     return "VAR_LONG";
  case VAR_NAME:     return "VAR_NAME";
  case VAR_VEC_HEAD: return "VAR_VEC_HEAD";
  case VAR_VEC_RC:   return "VAR_VEC_RC";
  case VAR_COLOUR:   return "VAR_COLOUR";
  case VAR_2D:       return "VAR_2D";
  default: return "unknown seni_var type";
  }
}

i32 var_vector_length(seni_var *var)
{
  if (var->type != VAR_VEC_HEAD) {
    return 0;
  }

  i32 len = 0;
  seni_var *v = var->value.v;
  if (v->type != VAR_VEC_RC) {
    return 0;
  }
  v = v->next;
  
  while (v != NULL) {
    len++;
    v = v->next;
  }

  return len;
}

void pretty_print_seni_node(seni_node *node, char* msg)
{
  if (node == NULL) {
    printf("NULL NODE %s\n", msg);
    return;
  }
  printf("%s %s\n", node_type_name(node), msg);
}

void pretty_print_seni_var(seni_var *var, char* msg)
{
  if (var == NULL) {
    SENI_ERROR("pretty_print_seni_var: given NULL");
    return;
  }
  
  char *type = var_type_name(var);
  seni_value_in_use using = get_value_in_use(var->type);

  switch(using) {
  case USE_I:
    if (var->type == VAR_COLOUR) {
      printf("%s: %s : %d (%.2f, %.2f, %.2f, %.2f)\n", msg, type, var->value.i,
             var->f32_array[0], var->f32_array[1], var->f32_array[2], var->f32_array[3]);
    } else {
      printf("%s: %s : %d\n", msg, type, var->value.i);
    }
    break;
  case USE_F:
    printf("%s: %s : %.2f\n", msg,  type, var->value.f);
    break;
  case USE_L:
    printf("%s: %s : %llu\n", msg, type, (long long unsigned int)(var->value.l));
    break;
  case USE_V:
    if (var->type == VAR_VEC_HEAD) {
      seni_var *rc = var->value.v;
      printf("%s: %s : length %d ref_count: %d\n", msg, type, var_vector_length(var), rc->value.ref_count);
    } else {
      printf("%s: %s\n", msg,  type);
    }
    break;
  }
}

seni_node *safe_next(seni_node *expr)
{
  seni_node *sibling = expr->next;
  while(sibling && (sibling->type == NODE_WHITESPACE ||
                    sibling->type == NODE_COMMENT)) {
    sibling = sibling->next;
  }

  return sibling;
}

seni_node *safe_prev(seni_node *expr)
{
  seni_node *sibling = expr->prev;
  while(sibling && (sibling->type == NODE_WHITESPACE ||
                    sibling->type == NODE_COMMENT)) {
    sibling = sibling->prev;
  }

  return sibling;
}

i32 var_as_int(seni_var *var)
{
  seni_value_in_use using = get_value_in_use(var->type);

  if (using == USE_I) {
    return var->value.i;
  } else if (using == USE_F) {
    return (i32)(var->value.f);
  } else {
    SENI_ERROR("var_as_int given unconvertable seni_var");
  }

  return -1;
}

f32 var_as_float(seni_var *var)
{
  seni_value_in_use using = get_value_in_use(var->type);

  if (using == USE_I) {
    return (f32)(var->value.i);
  } else if (using == USE_F) {
    return var->value.f;
  } else {
    SENI_ERROR("var_as_float given unconvertable seni_var");
  }

  return -1.0f;
}

u64 var_as_long(seni_var *var)
{
  seni_value_in_use using = get_value_in_use(var->type);

  if (using == USE_L) {
    return var->value.l;
  } else if (using == USE_I) {
    SENI_ERROR("Converting seni_var.value.i to u64");             // todo: should be a SENI_WARN
    return (u64)(var->value.i);
  } else {
    SENI_ERROR("var_as_long given unconvertable seni_var");
  }

  return 0L;
}

void i32_as_var(seni_var *out, i32 i)
{
  out->type = VAR_INT;
  out->value.i = i;
}

void f32_as_var(seni_var *out, f32 f)
{
  out->type = VAR_FLOAT;
  out->value.f = f;
}

void colour_as_var(seni_var *out, seni_colour *c)
{
  out->type = VAR_COLOUR;

  out->value.i = (i32)(c->format);

  out->f32_array[0] = c->element[0];
  out->f32_array[1] = c->element[1];
  out->f32_array[2] = c->element[2];
  out->f32_array[3] = c->element[3];  
}

// returns the next available seni_var that the calling code can write to
seni_var *stack_push(seni_vm *vm)
{
  seni_var *var = &(vm->stack[vm->sp]);
  vm->sp++;
  return var;
}

seni_var *stack_pop(seni_vm *vm)
{
  if (vm->sp == 0) {
    return NULL;
  }
  
  vm->sp--;
  return &(vm->stack[vm->sp]);
}

seni_var *stack_peek(seni_vm *vm)
{
  if (vm->sp == 0) {
    return NULL;
  }
  return &(vm->stack[vm->sp - 1]);
}

// **************************************************
// Program
// **************************************************

#define STR(x) #x
#define XSTR(x) STR(x)

char *opcode_name(seni_opcode opcode)
{
  char *names[] = {
#define OPCODE(name,_) STR(name),
#include "seni_opcodes.h"
#undef OPCODE
  };

  return names[opcode];
}

i32 opcode_offset[] = {
#define OPCODE(_,offset) offset,
#include "seni_opcodes.h"
#undef OPCODE
};

seni_program *program_allocate(i32 code_max_size)
{
  seni_program *program = (seni_program *)calloc(1, sizeof(seni_program));

  program->code = (seni_bytecode *)calloc(code_max_size, sizeof(seni_bytecode));
  program->code_max_size = code_max_size;
  program->code_size = 0;
  program->opcode_offset = 0;

  return program;
}

void program_free(seni_program *program)
{
  free(program->code);
  free(program);
}

void var_move(seni_var *dest, seni_var *src);

seni_bytecode *program_emit_opcode(seni_program *program, seni_opcode op, seni_var *arg0, seni_var *arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  var_move(&(b->arg0), arg0);
  var_move(&(b->arg1), arg1);

  program->opcode_offset += opcode_offset[op];

  return b;
}

// emits an <opcode, i32, i32> triplet
seni_bytecode *program_emit_opcode_i32(seni_program *program, seni_opcode op, i32 arg0, i32 arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  i32_as_var(&(b->arg0), arg0);
  i32_as_var(&(b->arg1), arg1);

  program->opcode_offset += opcode_offset[op];

  return b;
}

// emits an <opcode, i32, f32> triplet
seni_bytecode *program_emit_opcode_i32_f32(seni_program *program, seni_opcode op, i32 arg0, f32 arg1)
{
  if (program->code_size >= program->code_max_size) {
    SENI_ERROR("%s %d program has reached max size", __FILE__, __LINE__);
    return NULL;
  }
  
  seni_bytecode *b = &(program->code[program->code_size++]);
  b->op = op;
  i32_as_var(&(b->arg0), arg0);
  f32_as_var(&(b->arg1), arg1);

  program->opcode_offset += opcode_offset[op];

  return b;
}

char *memory_segment_name(seni_memory_segment_type segment)
{
  switch(segment) {
  case MEM_SEG_ARGUMENT:
    return "ARG";
  case MEM_SEG_LOCAL:
    return "LOCAL";
  case MEM_SEG_GLOBAL:
    return "GLOBAL";
  case MEM_SEG_CONSTANT:
    return "CONST";
  case MEM_SEG_VOID:
    return "VOID";
  }
  return "UNKNOWN";
}

void pretty_print_bytecode(i32 ip, seni_bytecode *b)
{
  if (b->op == LOAD || b->op == STORE || b->op == DEC_RC || b->op == INC_RC) {
    printf("%d\t%s\t%s\t",
           ip,
           opcode_name(b->op),
           memory_segment_name((seni_memory_segment_type)b->arg0.value.i));

    seni_value_in_use using = get_value_in_use(b->arg1.type);
    switch(using) {
    case USE_I:
      if (b->arg1.type == VAR_COLOUR) {
        printf("colour: %d (%.2f, %.2f, %.2f, %.2f)\n", b->arg1.value.i,
               b->arg1.f32_array[0], b->arg1.f32_array[1], b->arg1.f32_array[2], b->arg1.f32_array[3]);
      } else {
        printf("%d\n", b->arg1.value.i);
      }
      break;
    case USE_F:
      printf("%.2f\n", b->arg1.value.f);
      break;
    case USE_L:
      printf("%llu\n", (long long unsigned int)(b->arg1.value.l));
      break;
    case USE_V:
      if (b->arg1.type == VAR_VEC_HEAD) {
        printf("[..]len %d\n", var_vector_length(&(b->arg1)));
      } else {
        printf("[..]\n");
      }
      break;
    default:
      printf("unknown type\n");
    }
    
  } else if (b->op == JUMP_IF || b->op == JUMP) {
    printf("%d\t%s\t",
           ip,
           opcode_name(b->op));
    if (b->arg0.value.i > 0) {
      printf("+%d\n", b->arg0.value.i);
    } else if (b->arg0.value.i < 0) {
      printf("%d\n", b->arg0.value.i);
    } else {
      printf("WTF!\n");
    }
  } else if (b->op == NATIVE) {
    printf("%d\t%s\t%d\t%d\n",
           ip,
           opcode_name(b->op),
           b->arg0.value.i,
           b->arg1.value.i);    
  } else if (b->op == CALL_0 || b->op == CALL) {
    printf("%d\t%s\t%d\t%d\n",
           ip,
           opcode_name(b->op),
           b->arg0.value.i,
           b->arg1.value.i);
  } else if (b->op == PILE) {
    printf("%d\t%s\t%d\n",
           ip,
           opcode_name(b->op),
           b->arg0.value.i);
  } else {
    printf("%d\t%s\n", ip, opcode_name(b->op));
  }  
}

void pretty_print_program(seni_program *program)
{
  for (i32 i = 0; i < program->code_size; i++) {
    seni_bytecode *b = &(program->code[i]);
    pretty_print_bytecode(i, b);
  }
  printf("\n");
}

seni_env *env_construct()
{
  seni_env *e = (seni_env *)calloc(1, sizeof(seni_env));
  return e;
}

void env_free(seni_env *e)
{
  free(e);
}

// **************************************************
// Slab Info
// **************************************************

void slab_reset(seni_slab_info *slab_info)
{
  slab_info->get_count = 0;
  slab_info->return_count = 0;

  slab_info->delta = 0;
}

void slab_full_reset(seni_slab_info *slab_info)
{
  slab_reset(slab_info);

  slab_info->size = 0;
  slab_info->high_water_mark = 0;
}

void slab_get(seni_slab_info *slab_info)
{
  slab_info->get_count++;
  slab_info->delta++;
  slab_info->high_water_mark = max_i32(slab_info->high_water_mark, slab_info->delta);
}

void slab_return(seni_slab_info *slab_info, char *msg)
{
  slab_info->return_count++;
  slab_info->delta--;

  if (slab_info->delta < 0) {
    SENI_ERROR("slab_return called more often than slab_get %s", msg);
  }
}

void slab_print(seni_slab_info *slab_info, char *message)
{
  printf("%s\tsize: %d\n", message, slab_info->size);
  printf("\t\tget_count %d\treturn_count %d\n", slab_info->get_count, slab_info->return_count);
  printf("\t\tdelta: %d\thigh_water_mark %d\n", slab_info->delta, slab_info->high_water_mark);
}

// **************************************************
// Virtual Machine
// **************************************************

seni_vm *vm_construct(i32 stack_size, i32 heap_size)
{
  seni_var *var;
  i32 base_offset = 0;
  i32 i;
  seni_vm *vm = (seni_vm *)calloc(1, sizeof(seni_vm));

  vm->render_data = NULL;

  vm->stack = (seni_var *)calloc(stack_size, sizeof(seni_var));
  vm->stack_size = stack_size;

  vm->global = base_offset;
  base_offset += MEMORY_GLOBAL_SIZE;

  vm->ip = 0;
  
  vm->fp = base_offset;
  var = &(vm->stack[vm->fp]);
  var->type = VAR_INT;
  var->value.i = vm->fp;

  // add some offsets so that the memory after fp matches a standard format
  base_offset++;                // the caller's frame pointer
  base_offset++;                // the caller's ip
  base_offset++;                // the num_args of the called function

  vm->local = base_offset;
  base_offset += MEMORY_LOCAL_SIZE;
  vm->sp = base_offset;

  vm->heap_slab = (seni_var *)calloc(heap_size, sizeof(seni_var));
  vm->heap_avail = NULL;
  slab_full_reset(&(vm->heap_slab_info));
  vm->heap_slab_info.size = heap_size;


  var = vm->heap_slab;
  for (i = 0; i < heap_size; i++) {
    var[i].allocated = false;
    DL_APPEND(vm->heap_avail, &(var[i]));
  }

  vm->matrix_stack = matrix_stack_construct();
  // add an identity matrix onto the stack so that further scale/rotate/translate ops can work
  seni_matrix *matrix = matrix_stack_push(vm->matrix_stack);
  matrix_identity(matrix);

  return vm;
}

void vm_free(seni_vm *vm)
{
  render_data_free(vm->render_data);
  matrix_stack_free(vm->matrix_stack);
  free(vm->stack);
  free(vm->heap_slab);
  free(vm);
}

void pretty_print_vm(seni_vm *vm, char* msg)
{
  printf("%s\tvm: fp:%d sp:%d ip:%d local:%d\n",
         msg,
         vm->fp,
         vm->sp,
         vm->ip,
         vm->local);

  seni_var *fp = &(vm->stack[vm->fp]);
  i32 onStackFP = (fp + 0)->value.i;
  i32 onStackIP = (fp + 1)->value.i;
  i32 onStackNumArgs = (fp + 2)->value.i;
  printf("\ton stack: fp:%d ip:%d numArgs:%d\n", onStackFP, onStackIP, onStackNumArgs);
}

bool vector_ref_count_decrement(seni_vm *vm, seni_var *vec_head);

seni_var *var_get_from_heap(seni_vm *vm)
{
  seni_var *head = vm->heap_avail;

  if (head != NULL) {
    DEBUG_INFO_GET_FROM_HEAP(vm);
    DL_DELETE(vm->heap_avail, head);
  } else {
    SENI_ERROR("no more vars in pool");
    return NULL;
  }

  if (head->allocated) {
    SENI_ERROR("how did an already allocated seni_var get on the heap?");
    pretty_print_seni_var(head, "ERROR: var_get_from_heap");
    return NULL;
  }

  head->allocated = true;

  head->next = NULL;
  head->prev = NULL;

  head->value.i = 0;
  head->type = VAR_INT;         // just make sure that it isn't VAR_VEC_HEAD from a previous allocation

  //pretty_print_seni_var(head, "getting");

  return head;
}

void var_return_to_heap(seni_vm *vm,  seni_var *var)
{
  if(var->allocated == false) {
    // in case of 2 bindings to the same variable
    // e.g. (define a [1 2]) (define b [3 4]) (setq a b)
    // a and b both point to [3 4]
    return;
  }

  DEBUG_INFO_RETURN_TO_HEAP(vm);

  if (var->type == VAR_VEC_HEAD) {
    bool res = vector_ref_count_decrement(vm, var);
    if(res == false) {
      SENI_ERROR("var_return_to_heap");
    }
  }

  // the var is part of an allocated list
  if (var->next != NULL) {
    var_return_to_heap(vm, var->next);
  }

  var->allocated = false;
  DL_APPEND(vm->heap_avail, var);
}
  
bool vector_ref_count_decrement(seni_vm *vm, seni_var *vec_head)
{
  seni_var *var_rc = vec_head->value.v;
  if (var_rc->type != VAR_VEC_RC) {
    SENI_ERROR("a VAR_VEC_HEAD that isn't pointing to a VAR_VEC_RC???");
    pretty_print_seni_var(vec_head, "vector_ref_count_decrement called on this???");
    return false;
  }

  var_rc->value.ref_count--;

  //printf("vector_ref_count_decrement %p: %d\n", var_rc, var_rc->value.ref_count);

  // decrement the ref counts of any nested vectors
  seni_var *element = var_rc->next;
  while (element != NULL) {
    if (element->type == VAR_VEC_HEAD) {
      vector_ref_count_decrement(vm, element);
    }    
    element = element->next;
  }
      
  if (var_rc->value.ref_count == 0) {
    var_return_to_heap(vm, var_rc);
  } else if (var_rc->value.ref_count < 0) {
    SENI_ERROR("vector_ref_count_decrement: ref_count is %d", var_rc->value.ref_count);
    return false;
  }

  return true;
}

void vector_ref_count_increment(seni_vm *vm, seni_var *vec_head)
{
  seni_var *var_rc = vec_head->value.v;
  if (var_rc->type != VAR_VEC_RC) {
    SENI_ERROR("a VAR_VEC_HEAD that isn't pointing to a VAR_VEC_RC %d???", vm->sp);
  }
  
  var_rc->value.ref_count++;

#ifdef WHAT_THE_FUCK_MAC_HACK
  /* 
     this printf should have absolutely no effect, but it does prevent some of the 
     unit tests failing on OSX
   */
  printf("");
#endif

  //printf("vector_ref_count_increment %p: %d\n", var_rc, var_rc->value.ref_count);
}

bool var_copy(seni_vm *vm, seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return true;
  }

  if (dest->type == VAR_VEC_HEAD) {
    bool res = vector_ref_count_decrement(vm, dest);
    if (res == false) {
      SENI_ERROR("var_copy - vector_ref_count_decrement failed");
      return false;
    }
  }

  dest->type = src->type;
  dest->f32_array[0] = src->f32_array[0];
  dest->f32_array[1] = src->f32_array[1];
  dest->f32_array[2] = src->f32_array[2];
  dest->f32_array[3] = src->f32_array[3];

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_L) {
    dest->value.l = src->value.l;
  } else if (using == USE_V) {
    if (src->type == VAR_VEC_HEAD) {
      dest->value.v = src->value.v;
      vector_ref_count_increment(vm, src);
    } else {
      SENI_ERROR("what the fuck?\n");
    }
  } else {
    SENI_ERROR("unknown seni_value_in_use for var_copy");
  }

  return true;
}

// copying the src onto a var that we're not using (e.g. the top of a stack)
// only the reference counts of the src should be updated
bool var_copy_onto_junk(seni_vm *vm, seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return true;
  }

  dest->type = src->type;
  dest->f32_array[0] = src->f32_array[0];
  dest->f32_array[1] = src->f32_array[1];
  dest->f32_array[2] = src->f32_array[2];
  dest->f32_array[3] = src->f32_array[3];

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_L) {
    dest->value.l = src->value.l;
  } else if (using == USE_V) {
    if (src->type == VAR_VEC_HEAD) {
      dest->value.v = src->value.v;
      vector_ref_count_increment(vm, src);
    } else {
      SENI_ERROR("what the fuck?\n");
    }
  } else {
    SENI_ERROR("unknown seni_value_in_use for var_copy_onto_junk");
  }

  return true;
}

// like a seni_var_copy without any modifications to the ref count
void var_move(seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return;
  }

  dest->type = src->type;
  dest->f32_array[0] = src->f32_array[0];
  dest->f32_array[1] = src->f32_array[1];
  dest->f32_array[2] = src->f32_array[2];
  dest->f32_array[3] = src->f32_array[3];

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_L) {
    dest->value.l = src->value.l;
  } else if (using == USE_V) {
    if (src->type == VAR_VEC_HEAD) {
      dest->value.v = src->value.v;
    } else {
      SENI_ERROR("what the fuck?\n");
    }
  } else {
    SENI_ERROR("unknown seni_value_in_use for var_move");
  }
}

// [ ] <<- this is the VAR_VEC_HEAD (value.v points to VAR_VEC_RC)
//  |
// [ ] <<- this is the VAR_VEC_RC (value.ref_count is used)
//  |
//  v  <<= the VAR_VEC_RC's next pointer points to the contents of the vector
// [4] -> [7] -> [3] -> [5] -> NULL  <<- these are seni_vars
//
void vector_construct(seni_vm *vm, seni_var *head)
{
  seni_var *rc = var_get_from_heap(vm);    // get a vec_rc
  if (rc == NULL) {
    SENI_ERROR("vector_construct");
    return;
  }
  
  rc->type = VAR_VEC_RC;
  rc->value.ref_count = 1;

  // assuming that it's ok to wipe out head->value.v
  head->type = VAR_VEC_HEAD;
  head->value.v = NULL;           // attach vec_rc to vec_head
  DL_APPEND(head->value.v, rc);
}

void append_to_vector_i32(seni_vm *vm, seni_var *head, i32 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("append_to_vector_i32");
    return;
  }
  
  v->type = VAR_INT;
  v->value.i = val;

  DL_APPEND(head->value.v, v);
}

void append_to_vector_f32(seni_vm *vm, seni_var *head, f32 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("append_to_vector_f32");
    return;
  }
  
  v->type = VAR_FLOAT;
  v->value.f = val;

  DL_APPEND(head->value.v, v);
}

void append_to_vector_u64(seni_vm *vm, seni_var *head, u64 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("append_to_vector_u64");
    return;
  }
  v->type = VAR_LONG;
  v->value.l = val;

  DL_APPEND(head->value.v, v);
}

void append_to_vector_col(seni_vm *vm, seni_var *head, seni_colour *col)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("append_to_vector_col");
    return;
  }

  colour_as_var(v, col);

  DL_APPEND(head->value.v, v);
}

bool append_to_vector(seni_vm *vm, seni_var *head, seni_var *val)
{
  // assuming that head is VAR_VEC_HEAD
  
  seni_var *child_value = var_get_from_heap(vm);
  if (child_value == NULL) {
    SENI_ERROR("cannot allocate child_value from pool");
    return false;
  }

  bool res = var_copy(vm, child_value, val);
  if (res == false) {
    SENI_ERROR("var_copy failed in append_to_vector");
    return false;
  }
  
  DL_APPEND(head->value.v, child_value);
  return true;
}

// **************************************************
// Compiler
// **************************************************


void clear_local_mappings(seni_program *program)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    program->local_mappings[i] = -1;
  }
}

i32 add_local_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    if(program->local_mappings[i] == -1) {
      program->local_mappings[i] = wlut_value;
      return i;
    }
  }

  SENI_ERROR("add_local_mapping failed: increase MEMORY_LOCAL_SIZE from %d", MEMORY_LOCAL_SIZE);  
  return -1;
}

i32 get_local_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_LOCAL_SIZE; i++) {
    if(program->local_mappings[i] == wlut_value) {
      return i;
    }
  }

  return -1;
}

void clear_global_mappings(seni_program *program)
{
  for (i32 i=0; i < MEMORY_GLOBAL_SIZE; i++) {
    program->global_mappings[i] = -1;
  }
}

i32 add_global_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_GLOBAL_SIZE; i++) {
    if(program->global_mappings[i] == -1) {
      program->global_mappings[i] = wlut_value;
      return i;
    }
  }

  SENI_ERROR("add_global_mapping failed: increase MEMORY_GLOBAL_SIZE from %d", MEMORY_GLOBAL_SIZE);
  return -1;
}

i32 get_global_mapping(seni_program *program, i32 wlut_value)
{
  for (i32 i=0; i < MEMORY_GLOBAL_SIZE; i++) {
    if(program->global_mappings[i] == wlut_value) {
      return i;
    }
  }

  return -1;
}

i32 get_argument_mapping(seni_fn_info *fn_info, i32 wlut_value)
{
  for (i32 i = 0; i < MAX_NUM_ARGUMENTS; i++) {
    if (fn_info->argument_offsets[i] == -1) {
      return -1;
    }
    if (fn_info->argument_offsets[i] == wlut_value) {
      return (i * 2) + 1;
    }
  }
  return -1;
}

seni_node *compile(seni_node *ast, seni_program *program);

i32 node_vector_length(seni_node *vector_node)
{
  i32 length = 0;
  for (seni_node *node = vector_node->value.first_child; node != NULL; node = safe_next(node)) {
    length++;
  }
  return length;
}

bool all_children_have_type(seni_node *parent, seni_node_type type)
{
  if (parent->type != NODE_VECTOR && parent->type != NODE_LIST) {
    SENI_ERROR("all_children_have_type need a vector or list");
    return false;
  }

  seni_node *child = parent->value.first_child;
  while (child != NULL) {
    if (child->type != type) {
      return false;
    }
    child = safe_next(child);
  }

  return true;
}

i32 count_children(seni_node *parent)
{
  if (parent->type != NODE_VECTOR && parent->type != NODE_LIST) {
    SENI_ERROR("count_children need a vector or list");
    return 0;
  }

  i32 count = 0;
  seni_node *child = parent->value.first_child;
  while (child != NULL) {
    count++;
    child = safe_next(child);
  }

  return count;
}

i32 pop_from_stack_to_memory(seni_program *program, seni_node *node, seni_memory_segment_type memory_segment_type)
{
  i32 address = -1;
  
  if (memory_segment_type == MEM_SEG_LOCAL) {
    address = get_local_mapping(program, node->value.i);
    if (address == -1) {
      address = add_local_mapping(program, node->value.i);
      if (address == -1) {
        // failed to allocate
        SENI_ERROR("pop_from_stack_to_memory: allocation failure");
      }
    }
    program_emit_opcode_i32(program, STORE, MEM_SEG_LOCAL, address);
  } else if (memory_segment_type == MEM_SEG_GLOBAL) {
    address = get_global_mapping(program, node->value.i);
    if (address == -1) {
      address = add_global_mapping(program, node->value.i);
    }
    program_emit_opcode_i32(program, STORE, MEM_SEG_GLOBAL, address);
  } else {
    SENI_ERROR("pop_from_stack_to_memory: unknown memory_segment_type: %d", memory_segment_type);
  }

  return address;
}

seni_node *compile_define(seni_node *ast, seni_program *program, seni_memory_segment_type memory_segment_type)
{
  seni_node *lhs_node = safe_next(ast);
  seni_node *value_node;
  i32 i, m;

  while (lhs_node != NULL) {

    value_node = safe_next(lhs_node);
    compile(value_node, program);

    if (lhs_node->type == NODE_NAME) {
      // define foo 10
      m = pop_from_stack_to_memory(program, lhs_node, memory_segment_type);
      if (m == -1) {
        SENI_ERROR("compile_define: allocation failure in define");
        return NULL;
      }
    } else if (lhs_node->type == NODE_VECTOR) {
      // define [a b] (something-that-returns-a-vector ...)

      // check if we can use the PILE opcode
      if (all_children_have_type(lhs_node, NODE_NAME)) {
        i32 num_children = count_children(lhs_node);

        // PILE will stack the elements in the rhs vector in order,
        // so the lhs values have to be popped in reverse order
        program_emit_opcode_i32(program, PILE, num_children, 0);
        program->opcode_offset += num_children - 1;

        seni_node *child = lhs_node->value.first_child;


        for (i = 1; i < num_children; i++) {
          child = safe_next(child);
        }
        for (i = 0; i < num_children; i++) {
          m = pop_from_stack_to_memory(program, child, memory_segment_type);
          if (m == -1) {
            SENI_ERROR("compile_define: allocation failure during destructure");
            return NULL;
          }
          child = safe_prev(child);
        }
        /*        
        while (child != NULL) {
          pop_from_stack_to_memory(program, child, memory_segment_type);
          child = safe_next(child);
        }
        */
        
        
      } else {
        // this may be recursive
        printf("todo: push each item onto stack using nth");
      }

    } else {
      SENI_ERROR("compile_define lhs should be a name or a list");
    }

    lhs_node = safe_next(value_node);
  }

  return NULL;
}


void compile_if(seni_node *ast, seni_program *program)
{
  // if (> 200 100) 12 24
  // ^
  seni_node *if_node = safe_next(ast);
  seni_node *then_node = safe_next(if_node);
  seni_node *else_node = safe_next(then_node); // could be NULL

  compile(if_node, program);
  // insert jump to after the 'then' node if not true
  i32 addr_jump_then = program->code_size;
  seni_bytecode *bc_jump_then = program_emit_opcode_i32(program, JUMP_IF, 0, 0);

  compile(then_node, program);

  if (else_node) {
    // insert a bc_jump_else opcode
    i32 addr_jump_else = program->code_size;
    seni_bytecode *bc_jump_else = program_emit_opcode_i32(program, JUMP, 0, 0);

    bc_jump_then->arg0.value.i = program->code_size - addr_jump_then;

    compile(else_node, program);

    bc_jump_else->arg0.value.i = program->code_size - addr_jump_else;
  } else {
    bc_jump_then->arg0.value.i = program->code_size - addr_jump_then;
  }
}

// compiles everything after the current ast point
void compile_rest(seni_node *ast, seni_program *program)
{
  ast = safe_next(ast);
  while (ast) {
    ast = compile(ast, program);
  }
}

void compile_math(seni_node *ast, seni_program *program, seni_opcode opcode)
{
  // + 3 4 5 6
  //
  // 1	LOAD	CONST	3.00
  // 2	LOAD	CONST	4.00
  // 3	ADD
  // 4	LOAD	CONST	5.00
  // 5	ADD
  // 6	LOAD	CONST	6.00
  // 7	ADD
  
  ast = safe_next(ast); // skip the opcode

  ast = compile(ast, program); // compile the first argument
  while (ast) {
    ast = compile(ast, program); // compile the next argument
    program_emit_opcode_i32(program, opcode, 0, 0);
  }
}

void compile_loop(seni_node *ast, seni_program *program)
{
  // (loop (x from: 0 to: 5) (+ 42 38))
  //
  // 0       LOAD    CONST   0
  // 1       STORE     LOCAL   0
  // 2       LOAD    LOCAL   0
  // 3       LOAD    CONST   5
  // 4       LT
  // 5       JUMP_IF +10
  // 6       LOAD    CONST   42
  // 7       LOAD    CONST   38
  // 8       ADD
  // 9       STORE     VOID    0
  // 10      LOAD    LOCAL   0
  // 11      LOAD    CONST   1
  // 12      ADD
  // 13      STORE     LOCAL   0
  // 14      JUMP    -12
  // 15      STOP
  
  seni_node *parameters_node = safe_next(ast);
  if (parameters_node->type != NODE_LIST) {
    SENI_ERROR("expected a list that defines loop parameters");
    return;
  }

  // the looping variable x
  seni_node *name_node = parameters_node->value.first_child;



  seni_node *from_node = NULL;
  seni_node *to_node = NULL;
  seni_node *increment_node = NULL;
  bool have_from = false;
  bool have_to = false;
  bool have_increment = false;
  

  seni_node *node = name_node;

  while (node) {
    node = safe_next(node);  // the label part
    if (node == NULL) {
      break;
    }
    if (node->value.i == g_keyword_iname_from) {
      have_from = true;
      from_node = safe_next(node);
    }
    if (node->value.i == g_keyword_iname_to) {
      have_to = true;
      to_node = safe_next(node);
    }
    if (node->value.i == g_keyword_iname_increment) {
      have_increment = true;
      increment_node = safe_next(node);
    }
    node = safe_next(node); // the value part
  }

  if (have_to == false) {
    SENI_ERROR("loop form requires a 'to' parameter");
    return;
  }

  // set looping variable x to 'from' value
  if (have_from) {
    compile(from_node, program);
  } else {
    // else default to 0
    program_emit_opcode_i32_f32(program, LOAD, MEM_SEG_CONSTANT, 0.0f);
  }

  i32 looper_address = pop_from_stack_to_memory(program, name_node, MEM_SEG_LOCAL);
  if (looper_address == -1) {
    SENI_ERROR("compile_loop: allocation failure");
    return;
  }

  // compare looping variable against exit condition
  // and jump if looping variable >= exit value
  i32 addr_loop_start = program->code_size;
  program_emit_opcode_i32(program, LOAD, MEM_SEG_LOCAL, looper_address);
  compile(to_node, program);
  program_emit_opcode_i32(program, LT, 0, 0);
  i32 addr_exit_check = program->code_size;
  seni_bytecode *bc_exit_check = program_emit_opcode_i32(program, JUMP_IF, 0, 0);


  i32 pre_body_opcode_offset = program->opcode_offset;

  // compile the body forms (woooaaaoohhh body form, body form for yoooouuuu)
  compile_rest(parameters_node, program);

  i32 post_body_opcode_offset = program->opcode_offset;
  i32 opcode_delta = post_body_opcode_offset - pre_body_opcode_offset;

  // pop off any values that the body might leave on the stack
  for(i32 i = 0;i < opcode_delta; i++) {
    program_emit_opcode_i32(program, STORE, MEM_SEG_VOID, 0);
  }

  // increment the looping variable
  program_emit_opcode_i32(program, LOAD, MEM_SEG_LOCAL, looper_address);

  if (have_increment) {
    compile(increment_node, program);
  } else {
    program_emit_opcode_i32_f32(program, LOAD, MEM_SEG_CONSTANT, 1.0f);
  }
  program_emit_opcode_i32(program, ADD, 0, 0);
  program_emit_opcode_i32(program, STORE, MEM_SEG_LOCAL, looper_address);

  // loop back to the comparison
  program_emit_opcode_i32(program, JUMP, -(program->code_size - addr_loop_start), 0);
  bc_exit_check->arg0.value.i = program->code_size - addr_exit_check;
}

void compile_on_matrix_stack(seni_node *ast, seni_program *program)
{
  program_emit_opcode_i32(program, MTX_LOAD, 0, 0);
  compile_rest(ast, program);
  program_emit_opcode_i32(program, MTX_STORE, 0, 0);
}

// returns the index into program->fn_info that represents this function
i32 get_fn_info_index(seni_node *node, seni_program *program)
{
  if (node->type != NODE_NAME) {
    SENI_ERROR("get_fn_info_index not given a name node");
    return -1;
  }

  i32 name = node->value.i;
  
  for (i32 i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    if (program->fn_info[i].active == false) {
      return -1;
    }
    if (program->fn_info[i].fn_name == name) {
      return i;
    }
  }

  SENI_ERROR("get_fn_info_index unable to find fn_info for a function");
  return -1;
}

seni_fn_info *get_fn_info(seni_node *node, seni_program *program)
{
  if (node->type != NODE_NAME) {
    return NULL;
  }

  i32 name = node->value.i;
  
  for (i32 i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    if (program->fn_info[i].active == false) {
      return NULL;
    }
    if (program->fn_info[i].fn_name == name) {
      return &(program->fn_info[i]);
    }
  }
  return NULL;
}


i32 index_of_keyword(const char *keyword, seni_word_lut *wl)
{
  for (i32 i = 0; i < wl->keyword_count; i++) {
    if (strcmp(keyword, wl->keyword[i]) == 0) {
      return KEYWORD_START + i; // the keywords have KEYWORD_START added onto their index
    }
  }

  return -1;
}

void register_top_level_fns(seni_node *ast, seni_program *program)
{
  i32 i;
  i32 num_fns = 0;
  
  // clear all fn data
  for (i = 0; i < MAX_TOP_LEVEL_FUNCTIONS; i++) {
    program->fn_info[i].active = false;
  }

  // register top level fns
  while (ast != NULL) {

    if (ast->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }      

    seni_node *fn_keyword = ast->value.first_child;
    if (!(fn_keyword->type == NODE_NAME && fn_keyword->value.i == g_keyword_iname_fn)) {
      ast = safe_next(ast);
      continue;
    }

    // (fn (add-up a: 0 b: 0) (+ a b))
    // get the name of the fn
    seni_node *name_and_params = safe_next(fn_keyword);
    if (name_and_params->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }

    seni_node *name = name_and_params->value.first_child;
    i32 name_value = name->value.i;

    // we have a named top-level fn declaration
    seni_fn_info *fn_info = &(program->fn_info[num_fns]);
    num_fns++;
    if (num_fns > MAX_TOP_LEVEL_FUNCTIONS) {
      SENI_ERROR("Script has more than %d top-level functions\n", MAX_TOP_LEVEL_FUNCTIONS);
      return;
    }

    fn_info->active = true;
    fn_info->index = num_fns - 1;
    fn_info->fn_name = name_value;

    // these will be filled in by compile_fn:
    fn_info->num_args = 0;
    for (i = 0; i < MAX_NUM_ARGUMENTS; i++) {
      fn_info->argument_offsets[i] = -1;
    }

    ast = safe_next(ast);
  }
}


void register_names_in_define(seni_node *lhs, seni_program *program)
{
  if (lhs->type == NODE_NAME) {
    // (define foo 42)
    i32 global_address = get_global_mapping(program, lhs->value.i);
    if (global_address == -1) {
      global_address = add_global_mapping(program, lhs->value.i);
    }
  } else if (lhs->type == NODE_LIST || lhs->type == NODE_VECTOR) {
    // (define [a b] (something))
    // (define [a [x y]] (something))
    seni_node *child = lhs->value.first_child;

    while (child != NULL) {
      register_names_in_define(child, program);
      child = safe_next(child);
    }
  }  
}

void register_top_level_defines(seni_node *ast, seni_program *program)
{
  // register top level fns
  while (ast != NULL) {

    if (ast->type != NODE_LIST) {
      ast = safe_next(ast);
      continue;
    }

    seni_node *define_keyword = ast->value.first_child;
    if (!(define_keyword->type == NODE_NAME && define_keyword->value.i == g_keyword_iname_define)) {
      ast = safe_next(ast);
      continue;
    }

    seni_node *lhs = safe_next(define_keyword);
    while (lhs != NULL) {
      register_names_in_define(lhs, program);
      lhs = safe_next(lhs); // points to the value
      lhs = safe_next(lhs); // points to the next define statement if there multiple
    }

    ast = safe_next(ast);
  }
}

/*
  invoking code will first CALL into the arg_address to setup the default values for all args
  the fn code will then return back to the invoking code
  invoking code will then overwrite specific data in arg memory
  invoking code will then CALL into the body_address
*/
void compile_fn(seni_node *ast, seni_program *program)
{
  // fn (adder a: 0 b: 0) (+ a b)

  clear_local_mappings(program);

  // (adder a: 0 b: 0)
  seni_node *signature = safe_next(ast);

  seni_node *fn_name = signature->value.first_child;
  seni_fn_info *fn_info = get_fn_info(fn_name, program);
  if (fn_info == NULL) {
    SENI_ERROR("Unable to find fn_info for function %d", fn_name->value.i);
    return;
  }

  program->current_fn_info = fn_info;

  // -------------
  // the arguments
  // -------------
  
  fn_info->arg_address = program->code_size;
  seni_node *args = safe_next(fn_name); // pairs of label/value declarations
  i32 num_args = 0;
  i32 counter = 0;
  i32 argument_offsets_counter = 0;
  while (args != NULL) {
    seni_node *label = args;
    seni_node *value = safe_next(label);

    // get_argument_mapping
    fn_info->argument_offsets[argument_offsets_counter++] = label->value.i;

    // push pairs of label+value values onto the args stack
    program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, label->value.i);
    program_emit_opcode_i32(program, STORE, MEM_SEG_ARGUMENT, counter++);

    compile(value, program);
    program_emit_opcode_i32(program, STORE, MEM_SEG_ARGUMENT, counter++);

    num_args++;
    args = safe_next(value);
  }

  fn_info->num_args = num_args;

  program_emit_opcode_i32(program, RET_0, 0, 0);

  // --------
  // the body
  // --------

  fn_info->body_address = program->code_size;

  // (+ a b)
  compile_rest(signature, program);

  // Don't need any STORE, MEM_SEG_VOID instructions as the RET will
  // pop the frame and blow the stack

  program_emit_opcode_i32(program, RET, 0, 0);

  program->current_fn_info = NULL;
}

void correct_function_addresses(seni_program *program)
{
  // go through the bytecode fixing up function call addresses

  seni_bytecode *bc = program->code;
  i32 fn_info_index, label_value;
  seni_fn_info *fn_info;

  for (i32 i = 0; i < program->code_size; i++) {
    // replace the temporarily stored index in the args of CALL and CALL_0 with the actual values
    if (bc->op == CALL) {
      fn_info_index = bc->arg0.value.i; 
      fn_info = &(program->fn_info[fn_info_index]);
      
      bc->arg0.value.i = fn_info->arg_address;
      bc->arg1.value.i = fn_info->num_args;
    }
    
    if (bc->op == CALL_0) {
      fn_info_index = bc->arg0.value.i; 
      fn_info = &(program->fn_info[fn_info_index]);
      
      bc->arg0.value.i = fn_info->body_address;
      bc->arg1.value.i = fn_info->num_args;
    }

    if (bc->op == PLACEHOLDER_DEC_RC || bc->op == PLACEHOLDER_STORE || bc->op == PLACEHOLDER_INC_RC) {
      if (bc->op == PLACEHOLDER_DEC_RC) {
        bc->op = DEC_RC;
      }
      if (bc->op == PLACEHOLDER_INC_RC) {
        bc->op = INC_RC;
      }
      if (bc->op == PLACEHOLDER_STORE) {
        bc->op = STORE;
      }

      // opcode's arg0 is the fn_info_index and arg1 is the label_value
      fn_info_index = bc->arg0.value.i; 
      fn_info = &(program->fn_info[fn_info_index]);
      label_value = bc->arg1.value.i;
      
      i32 data_index = get_argument_mapping(fn_info, label_value);
      bc->arg1.value.i = data_index;

      if (data_index != -1) {
        bc->arg0.value.i = MEM_SEG_ARGUMENT;
      } else {
        // otherwise this function was invoked with a parameter that is doesn't use
        // so just essentially turn these ops into no-ops
        bc->arg0.value.i = MEM_SEG_VOID;
      }
    }

    bc++;
  }
}

void compile_fn_invocation(seni_node *ast, seni_program *program, i32 fn_info_index)
{
  // ast == adder a: 10 b: 20

  // NOTE: we're filling in the CALL, CALL_0 opcodes with placeholder values
  // that will later be replaced by the actual addresses
  // doing it this way enables functions to call other functions that are declared later in the script

  // prepare the MEM_SEG_ARGUMENT with default values

  program_emit_opcode_i32(program, CALL, fn_info_index, fn_info_index);

  // overwrite the default arguments with the actual arguments given by the fn invocation
  seni_node *args = safe_next(ast); // pairs of label/value declarations
  while (args != NULL) {
    seni_node *label = args;
    seni_node *value = safe_next(label);

    // push value
    compile(value, program);
    program_emit_opcode_i32(program, PLACEHOLDER_DEC_RC, fn_info_index, label->value.i);
    program_emit_opcode_i32(program, PLACEHOLDER_STORE, fn_info_index, label->value.i);

    if (value->type != NODE_VECTOR) {
      // not an explicitly declared vector so increment it's rc
      program_emit_opcode_i32(program, PLACEHOLDER_INC_RC, fn_info_index, label->value.i);

      // explicitly declared vectors will have an rc of 1, when the function
      // returns this will be decremented and they will be returned to the heap
    }

    args = safe_next(value);
  }
  
  // call the body of the function
  program_emit_opcode_i32(program, CALL_0, fn_info_index, fn_info_index);
}

// ast is a NODE_VECTOR of length 2
//
void compile_2d(seni_node *ast, seni_program *program)
{
  for (seni_node *node = ast->value.first_child; node != NULL; node = safe_next(node)) {
    compile(node, program);
  }
  program_emit_opcode_i32(program, SQUISH2, 0, 0);
}

void compile_vector(seni_node *ast, seni_program *program)
{
  // pushing from the VOID means creating a new, empty vector
  program_emit_opcode_i32(program, LOAD, MEM_SEG_VOID, 0);

  for (seni_node *node = ast->value.first_child; node != NULL; node = safe_next(node)) {
    compile(node, program);
    program_emit_opcode_i32(program, APPEND, 0, 0);
  }
}

seni_node *compile_user_defined_name(seni_node *ast, seni_program *program, i32 iname)
{
  i32 local_mapping = get_local_mapping(program, iname);
  if (local_mapping != -1) {
    program_emit_opcode_i32(program, LOAD, MEM_SEG_LOCAL, local_mapping);
    return safe_next(ast);
  }

  // check arguments if we're in a function
  if (program->current_fn_info) {
    i32 argument_mapping = get_argument_mapping(program->current_fn_info, iname);
    if (argument_mapping != -1) {
      program_emit_opcode_i32(program, LOAD, MEM_SEG_ARGUMENT, argument_mapping);
      return safe_next(ast);
    }
  }

  i32 global_mapping = get_global_mapping(program, iname);
  if (global_mapping != -1) {
    program_emit_opcode_i32(program, LOAD, MEM_SEG_GLOBAL, global_mapping);
    return safe_next(ast);
  }

  // could be a keyword such as linear, ease-in etc
  if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {
    program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, iname);
    return safe_next(ast);
  }


  SENI_ERROR("compile_user_defined_name: unknown mapping for %d", iname);
  return safe_next(ast);
}

seni_node *compile(seni_node *ast, seni_program *program)
{
  seni_node *n;

  if (ast->type == NODE_LIST) {
    n = ast->value.first_child;

    i32 fn_info_index = get_fn_info_index(n, program);
    if (fn_info_index != -1) {
      compile_fn_invocation(n, program, fn_info_index);
    } else {
      compile(n, program);
    }
    
    return safe_next(ast);
  }
  if (ast->type == NODE_FLOAT) {
    program_emit_opcode_i32_f32(program, LOAD, MEM_SEG_CONSTANT, ast->value.f);
    return safe_next(ast);
  }
  if (ast->type == NODE_INT) {
    program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, ast->value.i);
    return safe_next(ast);
  }
  if (ast->type == NODE_VECTOR) {
    if (node_vector_length(ast) == 2) {
      compile_2d(ast, program);
    } else {
      compile_vector(ast, program);
    }
    return safe_next(ast);
  }
  if (ast->type == NODE_NAME) {

    i32 iname = ast->value.i;
    
    if (iname >= WORD_START && iname < WORD_START + MAX_WORD_LOOKUPS) { // a user defined name
      return compile_user_defined_name(ast, program, iname);
    } else if (iname >= KEYWORD_START && iname < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {

      if (iname == g_keyword_iname_define) {
        return compile_define(ast, program, MEM_SEG_LOCAL);
      } else if (iname == g_keyword_iname_if) {
        compile_if(ast, program);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_loop) {
        compile_loop(ast, program);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_on_matrix_stack) {
        compile_on_matrix_stack(ast, program);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_fn) {
        compile_fn(ast, program);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_plus) {
        compile_math(ast, program, ADD);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_minus) {
        // TODO: differentiate between neg and sub?
        compile_math(ast, program, SUB);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_mult) {
        compile_math(ast, program, MUL);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_divide) {
        compile_math(ast, program, DIV);        
        return safe_next(ast);
      } else if (iname == g_keyword_iname_equal) {
        compile_math(ast, program, EQ);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_lt) {
        compile_math(ast, program, LT);        
        return safe_next(ast);
      } else if (iname == g_keyword_iname_gt) {
        compile_math(ast, program, GT);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_and) {
        compile_math(ast, program, AND);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_or) {
        compile_math(ast, program, OR);
        return safe_next(ast);
      } else if (iname == g_keyword_iname_not) {
        compile_rest(ast, program);
        program_emit_opcode_i32(program, NOT, 0, 0);
        return safe_next(ast);
      } else {
        // look up the name as a user defined variable
        // normally get here when a script contains variables
        // that have the same name as common parameters.
        // e.g. r, g, b, alpha
        // or if we're passing a pre-defined argument value
        // e.g. linear in (bezier line-width-mapping: linear)
        return compile_user_defined_name(ast, program, iname);
      }

    } else if ( iname >= NATIVE_START && iname < NATIVE_START + MAX_NATIVE_LOOKUPS){
      // NATIVE

      // note: how to count the stack delta? how many pop voids are required?
      i32 num_args = 0;
      seni_node *args = safe_next(ast); // pairs of label/value declarations
      while (args != NULL) {
        seni_node *label = args;
        seni_node *value = safe_next(label);

        program_emit_opcode_i32(program, LOAD, MEM_SEG_CONSTANT, label->value.i);
        compile(value, program);

        num_args++;
        args = safe_next(value);
      }
      
      program_emit_opcode_i32(program, NATIVE, iname, num_args);

      // modify opcode_offset according to how many args were given
      program->opcode_offset -= (num_args * 2) - 1;
      
      
      return safe_next(ast);
    }
  }

  return safe_next(ast);
}

bool is_list_beginning_with(seni_node *ast, i32 index)
{
  if (ast->type != NODE_LIST) {
    return false;
  }      

  seni_node *keyword = ast->value.first_child;
  if (keyword->type == NODE_NAME && keyword->value.i == index) {
    return true;
  }

  return false;  
}


void compile_preamble_f32(seni_program *program, i32 iname, f32 value)
{
  program_emit_opcode_i32_f32(program, LOAD, MEM_SEG_CONSTANT, value);
  i32 address = get_global_mapping(program, iname);
  if (address == -1) {
    address = add_global_mapping(program, iname);
  }
  program_emit_opcode_i32(program, STORE, MEM_SEG_GLOBAL, address);
}

// NOTE: each entry in compile_preamble should have a corresponding entry here
void register_top_level_preamble(seni_program *program)
{
  add_global_mapping(program, g_keyword_iname_canvas_width);
  add_global_mapping(program, g_keyword_iname_canvas_height);
}

// NOTE: each entry should have a corresponding entry in register_top_level_preamble
void compile_preamble(seni_program *program)
{
  compile_preamble_f32(program, g_keyword_iname_canvas_width, 1000.0f);
  compile_preamble_f32(program, g_keyword_iname_canvas_height, 1000.0f);
}


// compiles the ast into bytecode for a stack based VM
//
void compiler_compile(seni_node *ast, seni_program *program)
{
  clear_global_mappings(program);
  clear_local_mappings(program);
  program->current_fn_info = NULL;

  // register top-level functions
  register_top_level_fns(ast, program);

  // register top-level defines
  register_top_level_preamble(program);
  register_top_level_defines(ast, program);

  seni_bytecode *start = program_emit_opcode_i32(program, JUMP, 0, 0);

  // compile the top-level functions
  seni_node *n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, g_keyword_iname_fn)) {
      n = compile(n, program);
    } else {
      n = safe_next(n);
    }
  }

  // compile the global defines common to all seni programs
  // (e.g. canvas/width)
  // this is where the program will start from
  start->arg0.type = VAR_INT;
  start->arg0.value.i = program->code_size;
  compile_preamble(program);

  // compile the top-level defines
  n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, g_keyword_iname_define)) {
      compile_define(n->value.first_child, program, MEM_SEG_GLOBAL);
      n = safe_next(n);
    } else {
      n = safe_next(n);
    }
  }

  // compile all other top-level forms
  n = ast;
  while (n != NULL) {
    if (is_list_beginning_with(n, g_keyword_iname_fn) == false &&
        is_list_beginning_with(n, g_keyword_iname_define) == false) {
      n = compile(n, program);
    } else {
      n = safe_next(n);
    }
  }
  
  program_emit_opcode_i32(program, STOP, 0, 0);

  // we can now update the addreses used by CALL and CALL_0
  correct_function_addresses(program);

  // printf("program compiled: %d lines\n", program->code_size);
}

// **************************************************
// VM bytecode interpreter
// **************************************************

// executes a program on a vm
// returns true if we reached a STOP opcode
bool vm_interpret(seni_vm *vm, seni_program *program)
{
  bool b1, b2;
  f32 f1, f2;
  seni_memory_segment_type memory_segment_type;
  seni_var *src, *dest, *tmp;
  seni_matrix *top, *matrix;

  register seni_bytecode *bc = NULL;
  register seni_var *v = NULL;
  register i32 ip = vm->ip;
  register i32 sp = vm->sp;
  register seni_var *stack_d = &(vm->stack[sp]);

  i32 num_args;
  i32 iname;
  i32 i;

  // isg_col = 0;

  // the function calling convention means that references to LOCAL variables after a
  // CALL need to hop-back down the frame pointers to the real local frame that they
  // should be referencing. (see notes.org: bytecode sequence when calling functions)
  //
  i32 hop_back = 0;
  i32 local, fp;

#define STACK_PEEK v = stack_d - 1
#define STACK_POP stack_d--; sp--; v = stack_d
#define STACK_PUSH v = stack_d; stack_d++; sp++

  DEBUG_INFO_RESET(vm);

  clock_t start, diff;
  start = clock();
  
  for (;;) {
    vm->opcodes_executed++;
    bc = &(program->code[ip++]);

#ifdef TRACE_PRINT_OPCODES
    pretty_print_bytecode(ip-1, bc); // 0-index the ip so that it matches the pretty print program output
#endif
    
    switch(bc->op) {
    case LOAD:
      STACK_PUSH;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_CONSTANT) {
        var_move(v, &(bc->arg1));
      } else if (memory_segment_type == MEM_SEG_ARGUMENT) {

        // if we're referencing an ARG in-between CALL and CALL_0 make sure we use the right frame
        // i.e. we're using the caller function's ARG, not the callee
        fp = vm->fp;
        for (i = 0; i < hop_back; i++) {
          fp = vm->stack[fp].value.i;    // go back a frame
        }
        
        src = &(vm->stack[fp - bc->arg1.value.i - 1]);
#ifdef TRACE_PRINT_OPCODES
        pretty_print_seni_var(src, "---");
        printf("--- hop_back is %d fp is %d\n", hop_back, fp);
#endif
        var_copy_onto_junk(vm, v, src);
      } else if (memory_segment_type == MEM_SEG_LOCAL) {

        // if we're referencing a LOCAL in-between CALL and CALL_0 make sure we use the right frame
        fp = vm->fp;
        for (i = 0; i < hop_back; i++) {
          fp = vm->stack[fp].value.i;    // go back a frame
        }
        local = fp + 3;         // get the correct frame's local
        
        src = &(vm->stack[local + bc->arg1.value.i]);
        var_copy_onto_junk(vm, v, src);
      } else if (memory_segment_type == MEM_SEG_GLOBAL) {
        src = &(vm->stack[vm->global + bc->arg1.value.i]);
        var_copy_onto_junk(vm, v, src);
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // pushing from the void. i.e. create this object

        // temp: for the moment just assume that any LOAD VOID
        // means creating a new vector object.

        // also note that the VAR_VEC_HEAD is a seni_var from the stack
        // so it should never be sent to the vm->heap_avail
        vector_construct(vm, v);
        
      } else {
        SENI_ERROR("LOAD: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case STORE:
      STACK_POP;

      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        dest = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        // check the current value of dest,
        var_move(dest, v);
#ifdef TRACE_PRINT_OPCODES
        pretty_print_seni_var(dest, "---");
        printf("--- fp is %d\n", vm->fp);
#endif        
      } else if (memory_segment_type == MEM_SEG_LOCAL) {
        dest = &(vm->stack[vm->local + bc->arg1.value.i]);
        // using a copy since we could have a define in a loop and so
        // the previously assigned value will need to be reference counted
        var_copy(vm, dest, v);

        // the stack no longer references the vector, so decrement the rc
        if (v->type == VAR_VEC_HEAD) {
          b1 = vector_ref_count_decrement(vm, v);
          if (b1 == false) {
            SENI_ERROR("POP MEM_SEG_LOCAL: vector_ref_count_decrement failed");
            return false;
          }
        }
        
      } else if (memory_segment_type == MEM_SEG_GLOBAL) {
        dest = &(vm->stack[vm->global + bc->arg1.value.i]);
        var_move(dest, v);
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // normally pop from the stack and lose the value
        // but if it's a vector then decrement its ref count
        if (v->type == VAR_VEC_HEAD) {
          b1 = vector_ref_count_decrement(vm, v);
          if (b1 == false) {
            SENI_ERROR("STORE MEM_SEG_VOID: vector_ref_count_decrement failed");
            return false;
          }
        }
      } else {
        SENI_ERROR("STORE: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case DEC_RC:
      // the var referenced by the bytecode is a default value for a function argument
      // it's going to be overwritten by a parameter that was specified by the calling
      // code.
      // We'll need to decrement it's ref count
      //
      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        dest = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        if (dest->type == VAR_VEC_HEAD) {
          b1 = vector_ref_count_decrement(vm, dest);
          if (b1 == false) {
            SENI_ERROR("DEC_RC: vector_ref_count_decrement failed");
            return false;
          }
        }        
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // no nothing
      } else {
        SENI_ERROR("DEC_RC: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case INC_RC:
      //
      memory_segment_type = (seni_memory_segment_type)bc->arg0.value.i;
      if (memory_segment_type == MEM_SEG_ARGUMENT) {
        dest = &(vm->stack[vm->fp - bc->arg1.value.i - 1]);
        if (dest->type == VAR_VEC_HEAD) {
          vector_ref_count_increment(vm, dest);
        }
      } else if (memory_segment_type == MEM_SEG_VOID) {
        // no nothing
      } else {
        SENI_ERROR("INC_RC: unknown memory segment type %d", bc->arg0.value.i);
      }
      break;

    case JUMP:
      ip--;
      ip += bc->arg0.value.i;
      break;

    case JUMP_IF:
      STACK_POP;

      // jump if the top of the stack is false
      if (v->value.i == 0) {
        ip--;
        ip += bc->arg0.value.i;
      }
      break;

    case CALL:
      num_args = bc->arg1.value.i;

      // make room for the labelled arguments
      for (i = 0; i < num_args * 2; i++) {
        STACK_PUSH;
      }
      
      fp = sp;

      // push the caller's fp
      STACK_PUSH;
      v->type = VAR_INT;
      v->value.i = vm->fp;

      // push ip
      STACK_PUSH;
      v->type = VAR_INT;
      v->value.i = ip;

      // push num_args
      STACK_PUSH;
      v->type = VAR_INT;
      v->value.i = num_args;

      vm->ip = bc->arg0.value.i;
      vm->fp = fp;
      vm->local = sp;

      // clear the memory that's going to be used for locals
      for (i = 0; i < MEMORY_LOCAL_SIZE; i++) {
        // setting all memory as VAR_INT will prevent any weird ref count
        // stuff when we deal with the RET opcodes later on
        vm->stack[sp].type = VAR_INT; 
        sp++;
      }

      stack_d = &(vm->stack[sp]);
      ip = vm->ip;
      
      vm->sp = sp;

#ifdef TRACE_PRINT_OPCODES
        printf("--- fp is %d\n", vm->fp);
#endif        
      break;

    case CALL_0:
      // like CALL but keep the existing frame and just update the ip and return ip
      
      // set the correct return ip
      vm->stack[vm->fp + 1].value.i = ip;

      // leap to a location
      ip = bc->arg0.value.i;
      vm->ip = ip;

      // we're now executing the body of the function so don't
      // hop back when we push any arguments or locals onto the stack
      hop_back = 0;
      break;

    case RET_0:
      // leap to the return ip
      vm->ip = vm->stack[vm->fp + 1].value.i;
      ip = vm->ip;

      hop_back++;

      break;
      
    case RET:
      // pop the frame
      //

      // grab whatever was the last value on the soon to be popped frame
      src = &(vm->stack[sp - 1]);
      if (src->type == VAR_VEC_HEAD) {
        vector_ref_count_increment(vm, src);
      }

      num_args = vm->stack[vm->fp + 2].value.i;

      // decrement ref count on any locally defined vectors
      for (i = 0; i < MEMORY_LOCAL_SIZE; i++) {
        tmp = &(vm->stack[vm->local + i]);
        if (tmp->type == VAR_VEC_HEAD) {
          b1 = vector_ref_count_decrement(vm, tmp);
          if (b1 == false) {
            SENI_ERROR("RET local vector: vector_ref_count_decrement failed");
            return false;
          }
        }
      }

      for (i = 0; i < num_args; i++) {
        tmp = &(vm->stack[vm->fp - ((i+1) * 2)]);
        if (tmp->type == VAR_VEC_HEAD) {
          b1 = vector_ref_count_decrement(vm, tmp);
          if (b1 == false) {
            SENI_ERROR("RET args: vector_ref_count_decrement failed");
            return false;
          }
        }
      }

      // update vm
      vm->sp = vm->fp - (num_args * 2);
      vm->ip = vm->stack[vm->fp + 1].value.i;
      vm->fp = vm->stack[vm->fp].value.i;
      vm->local = vm->fp + 3;

      // sync registers with vm
      ip = vm->ip;
      sp = vm->sp;
      stack_d = &(vm->stack[sp]);

      // copy the previous frame's top stack value onto the current frame's stack
      STACK_PUSH;
      var_move(v, src);

#ifdef TRACE_PRINT_OPCODES
        printf("--- fp is %d\n", vm->fp);
#endif        
      break;

    case NATIVE:
      iname = bc->arg0.value.i - NATIVE_START;
      num_args = bc->arg1.value.i;

      // sync vm with registers
      vm->sp = sp;

      native_function_ptr native_func = program->env->function_ptr[iname];
      seni_var var = native_func(vm, num_args);
      
      // move vm->sp below the arguments, and decrement the rc of any vectors
      for (i = 0; i < num_args; i++) {
        vm->sp -= 2;
        tmp = &(vm->stack[vm->sp + 1]);
        if (tmp->type == VAR_VEC_HEAD) {
          b1 = vector_ref_count_decrement(vm, tmp);
          if (b1 == false) {
            SENI_ERROR("NATIVE: vector_ref_count_decrement failed");
            return false;
          }

          // this is now off the stack, so blow away the vector head
          tmp->type = VAR_INT;
          tmp->value.i = 0;
        }
      }
      
      // put the return value at the top of the stack
      var_move(&(vm->stack[vm->sp++]), &var);
      
      // sync registers with vm
      sp = vm->sp;
      stack_d = &(vm->stack[sp]);

      break;

    case APPEND:
      // pops top two values: a value and a vector
      // appends the value onto the vector
      STACK_POP;
      src = v;                      // the seni_var to append onto the vector

      STACK_POP;
      // v is the vector
      if (v->type != VAR_VEC_HEAD) {
        SENI_ERROR("APPEND expects the 2nd item on the stack to be a vector\n");
        return false;
      }

      b1 = append_to_vector(vm, v, src); // note: this uses a copy, should it be a move instead?
      if (b1 == false) {
        SENI_ERROR("append_to_vector failed in APPEND");
        DEBUG_INFO_PRINT(vm);
        pretty_print_seni_var(v, "the vector");
        pretty_print_seni_var(src, "the item to append");
        return false;
      }

      STACK_PUSH;
      break;

    case PILE:
      num_args = bc->arg0.value.i;

      STACK_POP;

      if (num_args == 2 && v->type == VAR_2D) {
        // top of the stack is a var_2d

        f1 = v->f32_array[0];
        f2 = v->f32_array[1];
        
        STACK_PUSH;
        f32_as_var(v, f1);
        STACK_PUSH;
        f32_as_var(v, f2);
        
      } else {
        // top of the stack contains a vector
        // take num_args elements from the vector and push them onto the stack
        seni_var _vec;
        var_move(&_vec, v);
        src = _vec.value.v->next;
        for (i = 0; i < num_args; i++) {
          STACK_PUSH;
          var_copy_onto_junk(vm, v, src);
          src = src->next;
        }
        b1 = vector_ref_count_decrement(vm, &_vec);
        if (b1 == false) {
          SENI_ERROR("PILE: vector_ref_count_decrement failed");
          return false;
        }
      }
      
      break;

    case SQUISH2:
      STACK_POP;
      if (v->type != VAR_FLOAT) {
        SENI_ERROR("SQUISH2 expects a float - non float in 2nd element of vector");
        // was the seni code declaring a vector of length 2 that didn't contain floats?
        // e.g. (define z [LAB RGB])
        // when would we ever want this kind of code?
        return false;
      }
      f2 = v->value.f;

      STACK_POP;
      if (v->type != VAR_FLOAT) {
        SENI_ERROR("SQUISH2 expects a float - non float in 1st element of vector");
        return false;
      }
      f1 = v->value.f;

      STACK_PUSH;
      v->type = VAR_2D;
      v->value.i = 0;
      v->f32_array[0] = f1;
      v->f32_array[1] = f2;
      break;

    case MTX_LOAD:
      // note: should these just be normal functions and not opcodes?
      top = matrix_stack_peek(vm->matrix_stack);
      matrix = matrix_stack_push(vm->matrix_stack);
      matrix_copy(matrix, top);
      break;

    case MTX_STORE:
      matrix_stack_pop(vm->matrix_stack);
      break;

    case ADD:
      STACK_POP;
      f2 = v->value.f;
      
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 + f2;
      break;

    case SUB:
      STACK_POP;
      f2 = v->value.f;

      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 - f2;

      break;

    case MUL:
      STACK_POP;
      f2 = v->value.f;
      
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 * f2;
      break;

    case DIV:
      STACK_POP;
      f2 = v->value.f;
      
      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.f = f1 / f2;
      break;

    case EQ:
      STACK_POP;
      f2 = v->value.f;

      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->type = VAR_BOOLEAN;
      v->value.i = f1 == f2;
      break;

    case GT:
      STACK_POP;
      f2 = v->value.f;

      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->type = VAR_BOOLEAN;
      v->value.i = f1 > f2;
      break;

    case LT:
      STACK_POP;
      f2 = v->value.f;

      STACK_POP;
      f1 = v->value.f;

      STACK_PUSH;
      v->value.i = f1 < f2;
      v->type = VAR_BOOLEAN;
      break;

    case AND:
      STACK_POP;
      b2 = (bool)v->value.i;

      STACK_POP;
      b1 = (bool)v->value.i;

      STACK_PUSH;
      v->value.i = b1 && b2;
      v->type = VAR_BOOLEAN;
      break;
      
    case OR:
      STACK_POP;
      b2 = (bool)v->value.i;

      STACK_POP;
      b1 = (bool)v->value.i;

      STACK_PUSH;
      v->value.i = b1 || b2;
      v->type = VAR_BOOLEAN;
      break;
      
    case NOT:
      STACK_POP;
      b1 = (bool)v->value.i;

      STACK_PUSH;
      v->value.i = !b1;
      v->type = VAR_BOOLEAN;
      break;
      
    case STOP:
      vm->sp = sp;
      diff = clock() - start;
      vm->execution_time = diff * 1000 / CLOCKS_PER_SEC;
      return true;
    default:
      SENI_ERROR("Unhandled opcode: %s\n", opcode_name(bc->op));
      return false;
    }
  }
}
