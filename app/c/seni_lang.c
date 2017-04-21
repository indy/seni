#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include <stdio.h>              /* for debug only */
#include <math.h>

#include "seni_containers.h"
#include "seni_lang.h"

// for parsing
seni_node *consume_item();

// for interpreting
void var_return_to_pool(seni_var *var);
seni_var *eval(seni_env *env, seni_node *expr);


#define NUM_ENV_ALLOCATED 32
seni_env *g_envs;               /* doubly linked list used as a pool of seni_env structs */
i32 g_envs_used;

#define NUM_VAR_ALLOCATED 1024
seni_var *g_vars;               /* doubly linked list used as a pool of seni_var structs */

seni_debug_info g_debug_info;

void string_copy_len(char **dst, char *src, size_t len)
{
  char *c = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(c, src, len);
  c[len] = '\0';

  *dst = c;
}

// interpreting typedefs and globals
typedef struct keyword {
  seni_var *(*function_ptr)(seni_env *, seni_node *);
  char *name;
} keyword;

keyword g_keyword[MAX_KEYWORD_LOOKUPS];



/* returns 0 if not found */
i32 lookup_reserved_name(word_lut *wlut, char *string, size_t len)
{
  i32 i = 0;
  for (i = 0; i < wlut->keywords_count; i++) {
    char *name = wlut->keywords[i];
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
    if (name[j] == '\0' && found == true) {
      return i + KEYWORD_START; // add offset
    }
  }

  return 0;
}

i32 wlut_lookup_or_add(word_lut *wlut, char *string, size_t len)
{
  i32 reserved = lookup_reserved_name(wlut, string, len);
  if (reserved != 0) {
    return reserved;
  }
  
  i32 i = 0;
  for (i = 0; i < wlut->words_count; i++) {
    char *name = wlut->words[i];
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
    if (name[j] == '\0' && found == true) {
      return i;
    }
  }
  
  /* string is not in the table and there's no room for another entry */
  if (i >= MAX_WORD_LOOKUPS) {
    return -1;
  }

  // the string is not in the lookup table, so add it
  string_copy_len(&(wlut->words[i]), string, len);
  wlut->words_count++;

  return i;
}

char *wlut_lookup(word_lut *wlut, i32 index)
{
  if (index >= KEYWORD_START) {
    return wlut->keywords[index - KEYWORD_START];
  }
  return wlut->words[index];
}

void wlut_free_keywords(word_lut *wlut)
{
  for( int i = 0; i < MAX_KEYWORD_LOOKUPS; i++) {
    if (wlut->keywords[i]) {
      free(wlut->keywords[i]);
    }
    wlut->keywords[i] = 0;      
  }
  wlut->keywords_count = 0;
}

void wlut_free_words(word_lut *wlut)
{
  for( int i = 0; i < MAX_WORD_LOOKUPS; i++) {
    if (wlut->words[i]) {
      free(wlut->words[i]);
    }
    wlut->words[i] = 0;      
  }
  wlut->words_count = 0;
}

word_lut *wlut_allocate()
{
  word_lut *wl = (word_lut *)calloc(1, sizeof(word_lut));
  return wl;
}

void wlut_free(word_lut *wlut)
{
  wlut_free_words(wlut);
  wlut_free_keywords(wlut);
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
  return (c == ' ' || c == '\t' || c == '\n' || c == ',') ? true : false;
}

bool is_digit(char c)
{
  return (c >= '0' && c <= '9') ? true : false;
}

bool is_alpha(char c)
{
  return ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) ? true : false;
}

bool is_symbol(char c)
{
  return (c == '+' || c == '-' || c == '*' || c == '/' || c == '=' ||
          c == '!' || c == '@' || c == '#' || c == '$' || c == '%' ||
          c == '^' || c == '&' || c == '<' || c == '>' || c == '?') ? true : false;
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

bool is_boolean_true(char *s, size_t word_len)
{
  return word_len == 4 && s[0] == 't' && s[1] == 'r' && s[2] == 'u' && s[3] == 'e';
}

bool is_boolean_false(char *s, size_t word_len)
{
  return word_len == 5 && s[0] == 'f' && s[1] == 'a' && s[2] == 'l' && s[3] == 's' && s[4] == 'e';
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

seni_node *build_text_lookup_node_from_string(word_lut *wlut, seni_node_type type, char *string)
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

seni_node *build_text_lookup_node_of_length(word_lut *wlut, char **src, seni_node_type type, size_t len)
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

seni_node *consume_list(word_lut *wlut, char **src)
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

seni_node *consume_vector(word_lut *wlut, char **src)
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

seni_node *consume_bracket(word_lut *wlut, char **src)
{
  seni_node *node;
  seni_node *parameter_prefix = NULL;
  seni_node *c;
  
  (*src)++; // {
  
  while (1) {
    c = consume_item(wlut, src);
    if (c == NULL) {
      SENI_ERROR("unable to consume element of bracket");
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

  /* TODO: sanity check the parameter prefixes */
  /* 
  prefixParameters.forEach(pp => node.addParameterNodePrefix(pp));

  if (nodeType !== NodeType.BOOLEAN &&
      nodeType !== NodeType.INT &&
      nodeType !== NodeType.FLOAT &&
      nodeType !== NodeType.NAME &&
      nodeType !== NodeType.STRING &&
      nodeType !== NodeType.LIST &&
      nodeType !== NodeType.VECTOR) {
    console.log('whooops', tokens, node);
    return {error: `non-mutable node within curly brackets ${nodeType}`};
  }
  */
  
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

seni_node *consume_quoted_form(word_lut *wlut, char **src)
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

seni_node *consume_int(char **src)
{
  char *end_ptr;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_INT;
  node->value.i = (i32)strtoimax(*src, &end_ptr, 10);

  *src = end_ptr;
  
  return node;
}

seni_node *consume_float(char **src)
{
  char *end_ptr;
  
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_FLOAT;
  node->value.f = (f32)strtof(*src, &end_ptr);

  *src = end_ptr;
  
  return node;
}

seni_node *consume_boolean(char **src, bool val)
{
  seni_node *node = (seni_node *)calloc(1, sizeof(seni_node));
  node->type = NODE_BOOLEAN;
  node->value.i = val;

  if (val == true) {
    (*src) += 4;                /* 'true' */
  } else {
    (*src) += 5;                /* 'false' */
  }
  
  return node;
}


seni_node *consume_name(word_lut *wlut, char **src)
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

seni_node *consume_string(word_lut *wlut, char **src)
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

seni_node *consume_label(word_lut *wlut, char **src)
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

seni_node *consume_item(word_lut *wlut, char **src)
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
    return consume_bracket(wlut, src);
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

      if (is_boolean_true(*src, word_len)) {
        return consume_boolean(src, true);
      }

      if (is_boolean_false(*src, word_len)) {
        return consume_boolean(src, false);
      } 

      return consume_name(wlut, src);
    }
  }
  
  if (is_digit(c) || is_minus(c) || is_period(c)) {
    if (has_period(*src)) {
      return consume_float(src);
    } else {
      return consume_int(src);
    }
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
  case NODE_BOOLEAN:    return "NODE_BOOLEAN";
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

seni_node *parser_parse(word_lut *wlut, char *s)
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
  case VAR_FN:
    return USE_N;
  case VAR_VEC_HEAD:
    return USE_V;
  case VAR_VEC_RC:
    return USE_V;
  default:
    return USE_I;
  };
}

char *var_type_name(seni_var *var)
{
  switch(var->type) {
  case VAR_INT:      return "VAR_INT";
  case VAR_FLOAT:    return "VAR_FLOAT";
  case VAR_BOOLEAN:  return "VAR_BOOLEAN";
  case VAR_NAME:     return "VAR_NAME";
  case VAR_FN:       return "VAR_FN";
  case VAR_VEC_HEAD: return "VAR_VEC_HEAD";
  case VAR_VEC_RC:   return "VAR_VEC_RC";
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
  v = v->value.v;
  
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
  char *type = var_type_name(var);
  seni_value_in_use using = get_value_in_use(var->type);

#ifdef SENI_DEBUG_MODE
  switch(using) {
  case USE_I:
    printf("id:%d %s : %d %s\n", var->debug_id, type, var->value.i, msg);
    break;
  case USE_F:
    printf("id:%d %s : %.2f %s\n", var->debug_id, type, var->value.f, msg);
    break;
  case USE_N:
    printf("id:%d %s %s %s\n", var->debug_id, type, node_type_name(var->value.n), msg);
    break;
  case USE_V:
    printf("id:%d %s : length %d %s\n", var->debug_id, type, var_vector_length(var), msg);
    break;
  }
#else
  switch(using) {
  case USE_I:
    printf("%s : %d %s\n", type, var->value.i, msg);
    break;
  case USE_F:
    printf("%s : %.2f %s\n", type, var->value.f, msg);
    break;
  case USE_N:
    printf("%s %s %s\n", type, node_type_name(var->value.n), msg);
    break;
  case USE_V:
    printf("%s : length %d %s\n", type, var_vector_length(var), msg);
    break;
  }
#endif
}

int env_debug_available_env()
{
  seni_env *seni_env;
  int count = 0;

  DL_COUNT(g_envs, seni_env, count);

  return count;
}

int env_debug_available_var()
{
  seni_var *seni_var;
  int count = 0;

  DL_COUNT(g_vars, seni_var, count);

  return count;
}

/* called once at startup */
void env_allocate_pools(void)
{
  i32 i;
  
  g_envs = NULL;
  g_envs_used = 0;

  seni_env *env = (seni_env *)calloc(NUM_ENV_ALLOCATED, sizeof(seni_env));
  for (i = 0; i < NUM_ENV_ALLOCATED; i++) {
    DL_APPEND(g_envs, &(env[i]));
  }

  g_vars = NULL;
  //g_vars_used = 0;

  seni_var *var = (seni_var *)calloc(NUM_VAR_ALLOCATED, sizeof(seni_var));
  for (i = 0; i < NUM_VAR_ALLOCATED; i++) {
#ifdef SENI_DEBUG_MODE
    var[i].debug_id = i;
    var[i].debug_allocatable = true;
#endif
    var[i].allocated = false;
    DL_APPEND(g_vars, &(var[i]));
  }
}

/* called once at shutdown */
void env_free_pools(void)
{
  seni_env *env, *tmp;
  DL_FOREACH_SAFE(g_envs, env, tmp) {
    DL_DELETE(g_envs, env);
  }
  free(g_envs);

  seni_var *var, *tmp2;
  DL_FOREACH_SAFE(g_vars, var, tmp2) {
    DL_DELETE(g_vars, var);
  }
  free(g_vars);
}

seni_env *env_get_from_pool()
{
  seni_env *head = g_envs;

  if (head != NULL) {
    DL_DELETE(g_envs, head);

    head->outer = NULL;
    head->vars = NULL;
    head->prev = NULL;
    head->next = NULL;
  }
  
  return head;
}

void env_return_to_pool(seni_env *env)
{
  DL_APPEND(g_envs, env);
}

seni_var *var_get_from_pool()
{
  //printf("getting %d ", env_debug_available_var());

  g_debug_info.var_get_count++;
  
  seni_var *head = g_vars;

  if (head != NULL) {
    DL_DELETE(g_vars, head);
  } else {
    SENI_ERROR("no more vars in pool");
  }

  if (head->allocated == true) {
    SENI_ERROR("how did an already allocated seni_var get in the pool?");
    pretty_print_seni_var(head, "var_get_from_pool");
  }

  head->allocated = true;

  head->next = NULL;
  head->prev = NULL;

  head->ref_count = 0;

  //pretty_print_seni_var(head, "getting");

  return head;
}

void vector_ref_count_decrement(seni_var *vec_head)
{
  seni_var *var_rc = vec_head->value.v;
  if (var_rc->type != VAR_VEC_RC) {
    SENI_ERROR("a VAR_VEC_HEAD that isn't pointing to a VAR_VEC_RC???");
  }

  //printf ("ref count is %d\n", var_rc->ref_count);
  var_rc->ref_count--;
      
  if (var_rc->ref_count == 0) {
    var_return_to_pool(var_rc);
  }
}

void vector_ref_count_increment(seni_var *vec_head)
{
  seni_var *var_rc = vec_head->value.v;
  if (var_rc->type != VAR_VEC_RC) {
    SENI_ERROR("a VAR_VEC_HEAD that isn't pointing to a VAR_VEC_RC???");
  }
  var_rc->ref_count++;
}

void var_return_to_pool(seni_var *var)
{
  if(var->allocated == false) {
    // in case of 2 bindings to the same variable
    // e.g. (define a [1 2]) (define b [3 4]) (setq a b)
    // a and b both point to [3 4]
    return;
  }

  g_debug_info.var_return_count++;
  // pretty_print_seni_var(var, "returning");

#ifdef SENI_DEBUG_MODE
  if (var->debug_allocatable == false) {
    SENI_ERROR("trying to return a seni_var to the pool that wasnt originally from the pool");
  }
#endif

  if (var->type == VAR_VEC_HEAD) {
    vector_ref_count_decrement(var);
  }
  
  if (var->type == VAR_VEC_RC) {
    if (var->value.v != NULL) {
      var_return_to_pool(var->value.v);
    }
  }

  // the var is part of an allocated list
  if (var->next != NULL) {
    var_return_to_pool(var->next);
  }

  var->allocated = false;
  DL_APPEND(g_vars, var);
}

seni_env *get_initial_env(seni_buffer *buffer)
{
  seni_env *env = env_get_from_pool();
  env->buffer = buffer;

  return env;
}

seni_env *push_scope(seni_env *outer)
{
  seni_env *env = env_get_from_pool();
  if (env == NULL) {
    // error
    return NULL;
  }

  env->buffer = outer->buffer;
  env->outer = outer;
  env->vars = NULL;
  
  return env;
}

seni_env *pop_scope(seni_env *env)
{
  seni_env *outer_env = env->outer;

  // return all the vars back to the pool
  seni_var *var, *tmp;

  // HASH_ITER(hh, env->vars, var, tmp) {
  //   pretty_print_seni_var(var, "popping");
  // }  

  HASH_ITER(hh, env->vars, var, tmp) {
    HASH_DEL(env->vars, var);
    var_return_to_pool(var);
  }  

  env_return_to_pool(env);
  
  return outer_env;
}

seni_var *lookup_var_in_current_scope(seni_env *env, i32 var_id)
{
  seni_var *var = NULL;

  HASH_FIND_INT(env->vars, &var_id, var);

  return var;
}

seni_var *lookup_var(seni_env *env, i32 var_id)
{
  if (env == NULL) {
    return NULL;
  }

  seni_var *var = lookup_var_in_current_scope(env, var_id);

  if (var != NULL) {
    return var;
  }

  return lookup_var(env->outer, var_id);
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

/* adds a var to the env */
seni_var *get_binded_var(seni_env *env, i32 var_id)
{
  seni_var *var = NULL;

  // is the key already assigned to this env?
  HASH_FIND_INT(env->vars, &var_id, var);
  if (var != NULL) {
    // return the existing var so that it can be overwritten
    // printf("-- already in hashmap -- %d ", env_debug_available_var());
    return var;
  }

  var = var_get_from_pool();
  if (var == NULL) {
    SENI_ERROR("unable to get a var from the pool");
    return NULL;
  }

  var->id = var_id;
  HASH_ADD_INT(env->vars, id, var);

  return var;
}



void safe_var_copy(seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return;
  }

  if (dest->type == VAR_VEC_HEAD) {
    vector_ref_count_decrement(dest);
  }
  
  dest->type = src->type;

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_N) {
    dest->value.n = src->value.n;
  } else if (using == USE_V) {
    if (src->type == VAR_VEC_HEAD) {
      dest->value.v = src->value.v;
      vector_ref_count_increment(dest);
    } else {
      printf("what the fuck?\n");
    }
  }
}

seni_var *eval_all_nodes(seni_env *env, seni_node *body)
{
  seni_var *res = NULL;

  while (body) {
    res = eval(env, body);
    body = safe_next(body);
  }

  return res;
}

// adds the named parameters to the env only if the current top-level scope
// of the env doesn't already contain bindings to those values. (this saves
// eval'ing values that are only going to be re-written by explicit parameters)
//
// e.g (fn (foo a: 1 b: 2) (+ a b)) (foo a: 3)
//
// the 'a: 1' won't be eval'd since the eval_fn will have already bound a to 3
//
void add_named_parameters_to_env(seni_env *env, seni_node *named_params)
{
  while (named_params) {
    seni_node *name = named_params;
    if (name->type != NODE_LABEL) {
      SENI_ERROR("expected a label as the start of the name/value pair");
      return;
    }
    i32 name_i = name->value.i;

    named_params = safe_next(named_params);
    if (named_params == NULL) {
      SENI_ERROR("expected name value pairs");
      return;
    }

    if (lookup_var_in_current_scope(env, name_i) == NULL) {
      seni_var *var = eval(env, named_params);
      bind_var(env, name_i, var);
    }

    named_params = safe_next(named_params);
  }
}

seni_node *get_named_node(seni_node *params, i32 name)
{
  // assuming we're at the first named parameter
  seni_node *node = params;
  bool found = false;
  
  while(node) {
    if (node->type != NODE_LABEL) {
      SENI_ERROR("should never get here, all name/value parameters should start with a NODE_NAME");
      return NULL;
    }

    found = node->value.i == name;

    node = safe_next(node);     // node is at the value
    if (node == NULL) {
      SENI_ERROR("should never get here, there should always be pairs of name/value");
      return NULL;
    }

    if (found) {
      return node;
    }

    node = safe_next(node);     // node is at the next named parameter
  }

  return NULL;
}

// does the sequence of named args contain the given name
bool has_named_node(seni_node *params, i32 name)
{
  seni_node *val = get_named_node(params, name);
  return val != NULL;
}

seni_var *get_named_var(seni_env *env, seni_node *params, i32 name)
{
  seni_node *node = get_named_node(params, name);
  if (node == NULL) {
    // couldn't find the parameter
    return NULL;
  }

  seni_var *var = eval(env, node);

  return var;
}

f32 get_named_f32(seni_env *env, seni_node *params, i32 name, f32 default_value)
{
  f32 ret = default_value;
  
  seni_var *var = get_named_var(env, params, name);
  if (var == NULL) {
    return ret;
  }
  ret = var_as_float(var);
  
  return ret;
}

i32 get_named_i32(seni_env *env, seni_node *params, i32 name, i32 default_value)
{
  i32 ret = default_value;
  
  seni_var *var = get_named_var(env, params, name);
  if (var == NULL) {
    return ret;
  }
  ret = var_as_int(var);
  
  return ret;
}

void get_named_vec2(seni_env *env, seni_node *params, i32 name, f32 *out0, f32 *out1)
{
  seni_var *var = get_named_var(env, params, name);
  if (var) {
    if (var->type != VAR_VEC_HEAD) {
      return;
    }

    var_as_vec2(out0, out1, var);

    if (var->allocated) {
      var_return_to_pool(var);
    } else {
      // return the VEC_RC and the vector's contents to the pool
      // don't free var though since it's going to be the static
      // seni_var in the eval function
      var_return_to_pool(var->value.v);
    }
    
  }
}

void get_named_vec4(seni_env *env, seni_node *params, i32 name, f32 *out0, f32 *out1, f32 *out2, f32 *out3)
{
  seni_var *var = get_named_var(env, params, name);
  if (var) {
    if (var->type != VAR_VEC_HEAD) {
      return;
    }

    var_as_vec4(out0, out1, out2, out3, var);

    if (var->allocated) {
      var_return_to_pool(var);
    } else {
      // return the VEC_RC and the vector's contents to the pool
      // don't free var though since it's going to be the static
      // seni_var in the eval function
      var_return_to_pool(var->value.v);
    }
    
  }
}

seni_var *eval_fn(seni_env *env, seni_node *expr)
{
  seni_node *name = expr->value.first_child;

  // look up the name in the env
  seni_var *var = lookup_var(env, name->value.i);  // var == fn (foo b: 1 c: 2) (+ b c)

  // should be of type VAR_FN
  if (var->type != VAR_FN) {
    SENI_ERROR("eval_fn - function invocation leads to non-fn binding");
  }

  seni_env *fn_env = push_scope(env);
  
  seni_node *fn_expr = var->value.n;

  // fn_expr points to the 'fn' keyword

  seni_node *fn_name_and_args_list = safe_next(fn_expr); // (foo b: 1 c: 2)
  
  seni_node *fn_args = safe_next(fn_name_and_args_list->value.first_child); // b: 1 c: 2

  // Add the invoked parameter bindings to the function's locally scoped env
  //
  seni_node *invoke_node = safe_next(name);

  while (invoke_node != NULL) {
    seni_node *arg_binding = invoke_node;
    i32 arg_binding_name = arg_binding->value.i;

    invoke_node = safe_next(invoke_node);

    seni_var *invoke_parameter_value = eval(env, invoke_node); // note: eval using the original outer scope

    bind_var(fn_env, arg_binding_name, invoke_parameter_value);

    invoke_node = safe_next(invoke_node);
  }

  // add the labelled default parameters if none were explicitly given
  add_named_parameters_to_env(fn_env, fn_args);

  seni_node *fn_body = safe_next(fn_name_and_args_list);

  seni_var *res = eval_all_nodes(fn_env, fn_body);

  pop_scope(fn_env);

  return res;
}

seni_var *eval_list(seni_env *env, seni_node *expr)
{
  seni_var *var = eval(env, expr->value.first_child);

  if (var->type == VAR_NAME && (var->value.i & KEYWORD_START)) {
    i32 i = var->value.i - KEYWORD_START;
    return (*g_keyword[i].function_ptr)(env, expr->value.first_child);
  }

  // user defined function
  if (var->type == VAR_FN) {
    return eval_fn(env, expr);
  }

  if (!(var->type == VAR_NAME || var->type == VAR_FN)) {
    printf("fuuck - only named functions can be invoked at the moment %d\n", var->type);
    return NULL;
  }

  return NULL;
}

// [ ] <<- this is the VAR_VEC_HEAD
//  |
// [4] -> [7] -> [3] -> [5] -> NULL  <<- these are seni_vars
//
seni_var *append_to_vector(seni_var *head, seni_var *val)
{
  // assuming that head is VAR_VEC_HEAD
  
  seni_var *child_value = var_get_from_pool();
  if (child_value == NULL) {
    SENI_ERROR("cannot allocate child_value from pool");
    return NULL;
  }
  safe_var_copy(child_value, val);
  //pretty_print_seni_var(child_value, "child val");

  seni_var *vec_rc = head->value.v;
  
  DL_APPEND(vec_rc->value.v, child_value);

  return head;
}

void debug_vector(seni_var *head)
{
  printf("\nhead->type %d\n", head->type);
  if (head->type != VAR_VEC_HEAD) {
    SENI_ERROR("fooked");
  }

  printf("head address %p\n", (void *)head);
  printf("head->value.v %p\n", (void *)head->value.v);
  pretty_print_seni_var(head, "head value");

  seni_var *vec_rc = head->value.v;
  if (!vec_rc || vec_rc->type != VAR_VEC_RC) {
    SENI_ERROR("no rc var attached to vector head");
  }
  printf("vector reference count %d\n", vec_rc->ref_count);

  seni_var *cons = vec_rc->value.v;
  while (cons != NULL) {
    pretty_print_seni_var(cons, "cons value");
    cons = cons->next;
  }
  printf("cons is null\n\n");
}

seni_var *eval(seni_env *env, seni_node *expr)
{
  // a register like seni_var for holding intermediate values
  static seni_var reg;
#ifdef SENI_DEBUG_MODE
  reg.debug_allocatable = false;
#endif

  seni_var *v = NULL;

  if (expr == NULL) {
    // in case of non-existent else clause in if statement
    printf("TODO: can we get here?\n");
    return NULL;
  }

  if (expr->type == NODE_VECTOR) {

    seni_var head, *val, *vec;

    // don't use reg since the elements of the vector will call eval and overwrite it
    head.type = VAR_VEC_HEAD;

    // create and attach a VAR_VEC_RC to head
    seni_var *var_rc = var_get_from_pool();
    if (var_rc == NULL) {
      SENI_ERROR("unable to get a var from the pool");
      return NULL;
    }
    var_rc->type = VAR_VEC_RC;
    var_rc->ref_count = 0;
    var_rc->value.v = NULL;

    head.value.v = var_rc;


    for (seni_node *node = expr->value.first_child; node != NULL; node = safe_next(node)) {
      val = eval(env, node);
      
      vec = append_to_vector(&head, val);
      if (vec == NULL) {
        return NULL;
      }
    }

    reg.type = head.type;
    reg.value.v = head.value.v; // this is a list of seni_vars whose  next/prev pointers are being used
    return &reg;
  }

  if (expr->type == NODE_INT) {
    reg.type = VAR_INT;
    reg.value.i = expr->value.i;

    return &reg;
  }
  
  if (expr->type == NODE_FLOAT) {
    reg.type = VAR_FLOAT;
    reg.value.f = expr->value.f;

    return &reg;
  }
  
  if (expr->type == NODE_BOOLEAN) {
    reg.type = VAR_BOOLEAN;
    reg.value.i = expr->value.i;

    return &reg;
  }

  if (expr->type == NODE_NAME) {
    if (expr->value.i & KEYWORD_START) {
      reg.type = VAR_NAME;
      reg.value.i = expr->value.i;

      return &reg;
    }
    v = lookup_var(env, expr->value.i);

    return v;
  }

  if (expr->type == NODE_LABEL || expr->type == NODE_STRING) {
    reg.type = VAR_INT;
    reg.value.i = expr->value.i;
    
    return &reg;
  }

  if (expr->type == NODE_LIST) {
    return eval_list(env, expr);
  }

  return NULL;
}

seni_var *true_in_reg(seni_var *reg)
{
  reg->type = VAR_BOOLEAN;
  reg->value.i = 1;

  return reg;
}

seni_var *false_in_reg(seni_var *reg)
{
  reg->type = VAR_BOOLEAN;
  reg->value.i = 0;

  return reg;
}

i32 var_as_int(seni_var *var)
{
  seni_value_in_use using = get_value_in_use(var->type);

  if (using == USE_I) {
    return var->value.i;
  } else if (using == USE_F) {
    return (i32)(var->value.f);
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
  }

  return -1.0f;
}

void var_as_vec2(f32 *out0, f32 *out1, seni_var *var)
{
  int len = var_vector_length(var);
  if (len != 2) {
    return;
  }

  seni_var *rc = var->value.v;
  seni_var *v = rc->value.v;

  f32 f0 = var_as_float(v);
  v = v->next;
  f32 f1 = var_as_float(v);
  
  *out0 = f0;
  *out1 = f1;
}

void var_as_vec4(f32 *out0, f32 *out1, f32 *out2, f32 *out3, seni_var *var)
{
  int len = var_vector_length(var);
  if (len != 4) {
    return;
  }

  seni_var *rc = var->value.v;
  seni_var *v = rc->value.v;

  f32 f0 = var_as_float(v);
  v = v->next;
  f32 f1 = var_as_float(v);
  v = v->next;
  f32 f2 = var_as_float(v);
  v = v->next;
  f32 f3 = var_as_float(v);
  
  *out0 = f0;
  *out1 = f1;
  *out2 = f2;
  *out3 = f3;
}

seni_var *bind_var(seni_env *env, i32 name, seni_var *var)
{
  seni_var *sv = get_binded_var(env, name);

  safe_var_copy(sv, var);

  // pretty_print_seni_var(sv, "var");

  return sv;
}

seni_var *bind_var_to_int(seni_env *env, i32 name, i32 value)
{
  seni_var *sv = get_binded_var(env, name);

  sv->type = VAR_INT;
  sv->value.i = value;

  // pretty_print_seni_var(sv, "int");

  return sv;
}

seni_var *bind_var_to_float(seni_env *env, i32 name, f32 value)
{
  seni_var *sv = get_binded_var(env, name);

  sv->type = VAR_FLOAT;
  sv->value.f = value;

  // pretty_print_seni_var(sv, "float");

  return sv;
}

void string_copy(char **dst, char *src)
{
  size_t len = strlen(src);
  
  char *c = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(c, src, len);
  c[len] = '\0';

  *dst = c;
}

// NOTE: the keyword.name is pointing to memory that's managed by the word_lut
//
void declare_keyword(word_lut *wlut, char *name, seni_var *(*function_ptr)(seni_env *, seni_node *))
{
  string_copy(&(wlut->keywords[wlut->keywords_count]), name);
  g_keyword[wlut->keywords_count].name = wlut->keywords[wlut->keywords_count];
  g_keyword[wlut->keywords_count].function_ptr = function_ptr;
  wlut->keywords_count++;

  if (wlut->keywords_count > MAX_KEYWORD_LOOKUPS) {
    SENI_ERROR("cannot declare keyword - wlut is full");
  }
}

void declare_common_arg(word_lut *wlut, char *name, i32 *global_value)
{
  declare_keyword(wlut, name, NULL);
  i32 i = wlut_lookup_or_add(wlut, name, strlen(name));
  *global_value = i;
}

seni_var *evaluate(seni_env *env, seni_node *ast, bool hygenic_scope)
{
  if (hygenic_scope) {

    // evaluate the ast in a clean scope that's popped afterwards.
    // This is useful when debugging for seni_var leaks and we want
    // to compare g_debug_info.
    // (see debug_lang_interpret_mem in test.c). It's also neater in
    // concept since it doesn't leave a polluted env after being
    // called so it should be used by default

    seni_env *e = push_scope(env);
    {
      for (seni_node *n = ast; n != NULL; n = safe_next(n)) {
        eval(e, n);
      }
    }
    pop_scope(e);

    // can't return the result of eval since that will reference
    // a seni_var that has been returned to the pool with the call
    // to pop_scope
    //
    return NULL;

  } else {

    // eval with the given seni_env, any modifications made will
    // persist. Useful for unit testing when we want to get the
    // result of an evaluation. (Normal behaviour is to add vertex
    // data to a buffer a.k.a side-effects)

    seni_var *res = NULL;
    {
      for (seni_node *n = ast; n != NULL; n = safe_next(n)) {
        res = eval(env, n);
      }
    }

    return res;

  }
}

void debug_var_info(seni_env *env)
{
  env = NULL;
  printf("vars available: %d, get count: %d, return count: %d\n",
         env_debug_available_var(),
         g_debug_info.var_get_count,
         g_debug_info.var_return_count);
}

void debug_reset()
{
  g_debug_info.num_var_allocated = NUM_VAR_ALLOCATED;
  g_debug_info.num_env_allocated = NUM_ENV_ALLOCATED;

  g_debug_info.num_var_available = env_debug_available_var();

  g_debug_info.var_get_count = 0;
  g_debug_info.var_return_count = 0;
}

void fill_debug_info(seni_debug_info *debug_info)
{
  debug_info->num_var_allocated = g_debug_info.num_var_allocated;
  debug_info->num_env_allocated = g_debug_info.num_env_allocated;

  debug_info->num_var_available = env_debug_available_var();

  debug_info->var_get_count = g_debug_info.var_get_count;
  debug_info->var_return_count = g_debug_info.var_return_count;
}
