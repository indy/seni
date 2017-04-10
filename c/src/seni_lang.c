#include <string.h>
#include <stdlib.h>
#include <inttypes.h>
#include <stdio.h>              /* for debug only */
#include <math.h>

#include "seni_containers.h"
#include "seni_lang.h"

// a register like seni_var for holding intermediate values
seni_var g_reg;


// for parsing
seni_node *consume_item();

// for interpreting
seni_var *eval(seni_env *env, seni_node *expr);

#define G_ENVS_MAX 32
seni_env *g_envs;               /* doubly linked list used as a pool of seni_env structs */
i32 g_envs_used;

#define G_VARS_MAX 64
seni_var *g_vars;               /* doubly linked list used as a pool of seni_var structs */
i32 g_vars_used;

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
      /* error? */
      return NULL;
    }

    DL_APPEND(node->value.children, child);
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
      /* error? */
      return NULL;
    }

    DL_APPEND(node->value.children, child);
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
      /* error? */
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
      /* error? */
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
  DL_APPEND(node->value.children, quote_name);

  char *wst = " ";
  seni_node *ws = build_text_node_of_length(&wst, NODE_WHITESPACE, 1);
  DL_APPEND(node->value.children, ws);

  seni_node *child = consume_item(wlut, src);
  DL_APPEND(node->value.children, child);

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

char *parser_node_type_name(seni_node_type type)
{
  switch(type) {
  case NODE_LIST: return "NODE_LIST";
  case NODE_VECTOR: return "NODE_VECTOR";
  case NODE_INT: return "NODE_INT";
  case NODE_FLOAT: return "NODE_FLOAT";
  case NODE_NAME: return "NODE_NAME";
  case NODE_LABEL: return "NODE_LABEL";
  case NODE_STRING: return "NODE_STRING";
  case NODE_BOOLEAN: return "NODE_BOOLEAN";
  case NODE_WHITESPACE: return "NODE_WHITESPACE";
  case NODE_COMMENT: return "NODE_COMMENT";
  default: return "unknown node type";
  };
}

void parser_free_nodes(seni_node *nodes)
{
  seni_node *node = nodes;
  seni_node *next;

  while(node != NULL) {
    if (node->type == NODE_LIST && node->value.children) {
      parser_free_nodes(node->value.children);
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

    // printf("freeing node: %s %u\n", parser_node_type_name(node->type), (u32)node);
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
  seni_env *env;

  for (i = 0; i < G_ENVS_MAX; i++) {
    env = (seni_env *)malloc(sizeof(seni_env));
    env->outer = NULL;
    env->vars = NULL;
    env->prev = NULL;
    env->next = NULL;
    DL_APPEND(g_envs, env);
  }

  g_vars = NULL;
  g_vars_used = 0;
  seni_var *var;

  for (i = 0; i < G_VARS_MAX; i++) {
    var = (seni_var *)malloc(sizeof(seni_var));
    DL_APPEND(g_vars, var);
  }
}

/* called once at shutdown */
void env_free_pools(void)
{
  seni_env *env, *tmp;
  DL_FOREACH_SAFE(g_envs, env, tmp) {
    DL_DELETE(g_envs, env);
    free(env);
  }  

  seni_var *var, *tmp2;
  DL_FOREACH_SAFE(g_vars, var, tmp2) {
    DL_DELETE(g_vars, var);
    free(var);
  }  
}

seni_env *get_env_from_pool()
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

void return_env_to_pool(seni_env *env)
{
  DL_APPEND(g_envs, env);
}

seni_var *get_var_from_pool()
{
  seni_var *head = g_vars;

  if (head != NULL) {
    DL_DELETE(g_vars, head);
  }
  
  return head;
}

void return_var_to_pool(seni_var *var)
{
  DL_APPEND(g_vars, var);
}

seni_env *get_initial_env()
{
  return get_env_from_pool();
}

seni_env *push_scope(seni_env *outer)
{
  seni_env *env = get_env_from_pool();
  if (env == NULL) {
    // error
    return NULL;
  }

  env->outer = outer;
  env->vars = NULL;
  
  return env;
}

seni_env *pop_scope(seni_env *env)
{
  seni_env *outer_env = env->outer;

  // return all the vars back to the pool
  seni_var *var, *tmp;

  HASH_ITER(hh, env->vars, var, tmp) {
    HASH_DEL(env->vars, var);
    return_var_to_pool(var);
  }  

  return_env_to_pool(env);
  
  return outer_env;
}

/* adds a var to the env */
seni_var *add_var(seni_env *env, i32 var_id)
{
  seni_var *var = NULL;

  // is the key already assigned to this env?
  HASH_FIND_INT(env->vars, &var_id, var);
  if (var != NULL) {
    // return the existing var so that it can be overwritten
    return var;
  }

  var = get_var_from_pool();
  if (var == NULL) {
    // error
    return NULL;
  }
  
  var->id = var_id;
  HASH_ADD_INT(env->vars, id, var);

  return var;
}

seni_var *lookup_var(seni_env *env, i32 var_id)
{
  if (env == NULL) {
    return NULL;
  }

  seni_var *var = NULL;

  HASH_FIND_INT(env->vars, &var_id, var);
  if (var != NULL) {
    // return the existing var
    return var;
  }

  return lookup_var(env->outer, var_id);
}

seni_value_in_use get_value_in_use(seni_var_type type)
{
  if (type == VAR_FLOAT) {
    return USE_F;
  } else if (type == VAR_FN) {
    return USE_P;
  } else {
    return USE_I;
  }
}

seni_var_type node_type_to_var_type(seni_node_type type)
{
  switch(type) {
  case NODE_INT:
    return VAR_INT;
  case NODE_FLOAT:
    return VAR_FLOAT;
  case NODE_BOOLEAN:
    return VAR_BOOLEAN;
  case NODE_NAME:
    return VAR_NAME;
  default:
    return VAR_INT;
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

void safe_seni_var_copy(seni_var *dest, seni_var *src)
{
  dest->type = src->type;

  seni_value_in_use using = get_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_P) {
    dest->value.p = src->value.p;
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


// a: 3 b: 10 c: (+ 5 6)
void add_labelled_parameters_to_env(seni_env *env, seni_node *named_args)
{
  while (named_args) {
    seni_node *name = named_args;
    if (name->type != NODE_LABEL) {
      // error: expected a label as the start of the name/value pair
      return;
    }
    i32 name_i = name->value.i;

    named_args = safe_next(named_args);
    if (named_args == NULL) {
      // error expected name value pairs
      return;
    }
    
    seni_node *value = named_args;
    seni_var *var = eval(env, value);
    seni_var *env_var = add_var(env, name_i);

    safe_seni_var_copy(env_var, var);

    named_args = safe_next(named_args);
  }
}

// does the sequence of named args contain the given name
bool has_labelled_parameter(seni_node *named_args, i32 name)
{
  // assuming we're at the first named parameter
  seni_node *node = named_args;
  
  while(node) {
    if (node->type != NODE_LABEL) {
      // error should never get here, all name/value parameters should start with a NODE_NAME
      return false;
    }

    if (node->value.i == name) {
      return true;
    }

    node = safe_next(node);     // node is at the value
    if (node == NULL) {
      // error should never get here, there should always be pairs of name/value
      return false;
    }

    node = safe_next(node);     // node is at the next named parameter
  }

  return false;
}


seni_var *eval_fn(seni_env *env, seni_node *expr)
{
  seni_node *name = expr->value.children;

  // look up the name in the env
  seni_var *var = lookup_var(env, name->value.i);  // var == fn (foo b: 1 c: 2) (+ b c)

  // should be of type VAR_FN
  if (var->type != VAR_FN) {
    // error: eval_fn - function invocation leads to non-fn binding
  }

  seni_env *fn_env = push_scope(env);
  
  seni_node *fn_expr = var->value.p;

  // fn_expr points to the 'fn' keyword

  seni_node *fn_name_and_args_list = safe_next(fn_expr); // (foo b: 1 c: 2)
  
  seni_node *fn_args = safe_next(fn_name_and_args_list->value.children); // b: 1 c: 2

  add_labelled_parameters_to_env(fn_env, fn_args);

  // Add the invoked parameter bindings to the function's locally scoped env
  //
  seni_node *invoke_node = safe_next(name);

  while (invoke_node != NULL) {
    seni_node *arg_binding = invoke_node;

    invoke_node = safe_next(invoke_node);
    seni_var *invoke_parameter_value = eval(env, invoke_node); // note: eval using the original outer scope

    seni_var *invoke_parameter = add_var(fn_env, arg_binding->value.i);
    safe_seni_var_copy(invoke_parameter, invoke_parameter_value);

    invoke_node = safe_next(invoke_node);
  }

  seni_node *fn_body = safe_next(fn_name_and_args_list);

  seni_var *res = eval_all_nodes(fn_env, fn_body);

  pop_scope(fn_env);

  return res;
}

seni_var *eval_list(seni_env *env, seni_node *expr)
{
  seni_var *var = eval(env, expr->value.children);

  if (var->type == VAR_NAME && (var->value.i & KEYWORD_START)) {
    i32 i = var->value.i - KEYWORD_START;
    return (*g_keyword[i].function_ptr)(env, expr->value.children);
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



seni_var *eval(seni_env *env, seni_node *expr)
{
  seni_var *v = NULL;
  
  if (expr == NULL) {
    // in case of non-existent else clause in if statement
    printf("TODO: can we get here?\n");
    return NULL;
  }

  if (expr->type == NODE_INT) {
    g_reg.type = node_type_to_var_type(expr->type);
    g_reg.value.i = expr->value.i;
    return &g_reg;
  }
  
  if (expr->type == NODE_FLOAT) {
    g_reg.type = node_type_to_var_type(expr->type);
    g_reg.value.f = expr->value.f;
    return &g_reg;
  }
  
  if (expr->type == NODE_BOOLEAN) {
    g_reg.type = node_type_to_var_type(expr->type);
    g_reg.value.i = expr->value.i;
    return &g_reg;
  }

  if (expr->type == NODE_NAME) {
    if (expr->value.i & KEYWORD_START) {
      g_reg.type = node_type_to_var_type(expr->type);
      g_reg.value.i = expr->value.i;
      return &g_reg;
    }
    v = lookup_var(env, expr->value.i);
    return v;
  }

  if (expr->type == NODE_LABEL || expr->type == NODE_STRING) {
      g_reg.type = node_type_to_var_type(expr->type);
      g_reg.value.i = expr->value.i;
      return &g_reg;
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

i32 var_as_int(seni_var *v1)
{
  seni_value_in_use using = get_value_in_use(v1->type);

  if (using == USE_I) {
    return v1->value.i;
  } else if (using == USE_F) {
    return (i32)(v1->value.f);
  }

  return -1;
}

f32 var_as_float(seni_var *v1)
{
  seni_value_in_use using = get_value_in_use(v1->type);

  if (using == USE_I) {
    return (f32)(v1->value.i);
  } else if (using == USE_F) {
    return v1->value.f;
  }

  return -1.0f;
}

void bind_var_to_int(seni_env *env, i32 name, i32 value)
{
  seni_var *sv = add_var(env, name);
  sv->type = VAR_INT;
  sv->value.i = value;
}

void bind_var_to_float(seni_env *env, i32 name, f32 value)
{
  seni_var *sv = add_var(env, name);
  sv->type = VAR_FLOAT;
  sv->value.f = value;
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
    // error
  }
}

void declare_common_arg(word_lut *wlut, char *name, i32 *global_value)
{
  declare_keyword(wlut, name, NULL);
  i32 i = wlut_lookup_or_add(wlut, name, strlen(name));
  *global_value = i;
}

seni_var *evaluate(seni_env *env, word_lut *wl, seni_node *ast)
{
  seni_var *res = NULL;
  for (seni_node *n = ast; n != NULL; n = safe_next(n)) {
    res = eval(env, n);
  }

  return res;
}
