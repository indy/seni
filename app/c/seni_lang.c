#include "seni_lang.h"

#include "seni_bind.h"
#include "seni_colour.h"
#include "seni_config.h"
#include "seni_keyword_iname.h"
#include "seni_mathutil.h"
#include "seni_matrix.h"
#include "seni_parser.h"
#include "seni_prng.h"
#include "seni_render_packet.h"
#include "seni_text_buffer.h"
#include "seni_vm_compiler.h"

#include <string.h>
#include <stdlib.h>
#include <inttypes.h>

#include "lib/utlist.h"

#include "seni_pool_macro.h"


// required by SENI_POOL macro
void var_cleanup(seni_var *var)
{
  var->value.v = NULL;
}

SENI_POOL(seni_var, var)

struct seni_var_pool *g_var_pool;


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

// called after a script has been executed
void wlut_reset_words(seni_word_lut *wlut)
{
  wlut_free_words(wlut);
  // leave keywords and natives since they aren't mutable
}

char *wlut_get_word(seni_word_lut *word_lut, i32 iword)
{
  if (iword < word_lut->word_count) {
    return word_lut->word[iword];
  }
  return "UNKNOWN WORD";
}

char *wlut_reverse_lookup(seni_word_lut *word_lut, i32 iword)
{
  if (iword < word_lut->word_count) {
    return word_lut->word[iword];
  }
  if (iword >= KEYWORD_START && iword < KEYWORD_START + MAX_KEYWORD_LOOKUPS) {
    return word_lut->keyword[iword - KEYWORD_START];
  }
  if (iword >= NATIVE_START && iword < NATIVE_START + MAX_NATIVE_LOOKUPS) {
    return word_lut->native[iword - NATIVE_START];
  }
  return "UNKNOWN WORD";
}

void wlut_pretty_print(char *msg, seni_word_lut *word_lut)
{
  SENI_PRINT("%s native_count: %d", msg, word_lut->native_count);
  SENI_PRINT("%s keyword_count: %d", msg, word_lut->keyword_count);
  SENI_PRINT("%s word_count: %d", msg, word_lut->word_count);
}

seni_value_in_use get_node_value_in_use(seni_node_type type)
{
  switch(type) {
  case NODE_LIST:
    return USE_FIRST_CHILD; // ???
    break;
  case NODE_VECTOR:
    return USE_FIRST_CHILD; // ???
    break;
  case NODE_INT:
    return USE_I;
    break;
  case NODE_FLOAT:
    return USE_F;
    break;
  case NODE_NAME:
    return USE_I;
    break;
  case NODE_LABEL:
    return USE_I;
    break;
  case NODE_STRING:
    return USE_I;
    break;
  case NODE_WHITESPACE:
    return USE_S;
    break;
  case NODE_COMMENT:
    return USE_S;
    break;
  }

  return USE_UNKNOWN;
}

// returns the first meaningful (non-whitespace, non-comment) node
seni_node *safe_first(seni_node *expr)
{
  if (expr == NULL) {
    return NULL;
  }
  if (expr->type != NODE_WHITESPACE && expr->type != NODE_COMMENT) {
    return expr;
  }

  return safe_next(expr);
}

seni_node *safe_first_child(seni_node *expr)
{
  if (get_node_value_in_use(expr->type) != USE_FIRST_CHILD) {
    SENI_ERROR("calling safe_first_child on a node that doesn't have a valid first child");
    return NULL;
  }

  seni_node *n = safe_first(expr->value.first_child);

  return n;
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

void node_pretty_print(char* msg, seni_node *node, seni_word_lut *word_lut)
{
  if (node == NULL) {
    SENI_ERROR("node_pretty_print: given NULL");
    return;
  }
  
  char *type = node_type_name(node);
  seni_value_in_use using = get_node_value_in_use(node->type);

  switch(using) {
  case USE_UNKNOWN:
    SENI_PRINT("%s: UNKNOWN %s", msg,  type);
    break;
  case USE_I:
    if (word_lut != NULL &&
        (node->type == NODE_NAME || node->type == NODE_LABEL || node->type == NODE_STRING)) {
      SENI_PRINT("%s: %s : %s (%d)", msg, type, wlut_reverse_lookup(word_lut, node->value.i), node->value.i);
    } else {
      SENI_PRINT("%s: %s : %d", msg, type, node->value.i);
    }
    break;
  case USE_F:
    SENI_PRINT("%s: %s : %.2f", msg,  type, node->value.f);
    break;
  case USE_L:
    SENI_PRINT("%s: L %s", msg,  type);
    break;
  case USE_V:
    SENI_PRINT("%s: V %s", msg,  type);
    break;
  case USE_S:
    SENI_PRINT("%s: %s", msg,  type);
    break;
  case USE_FIRST_CHILD:
    SENI_PRINT("%s: %s", msg,  type);
    break;
  default:
    SENI_ERROR("unknown using value for a seni_node: %d", using);
  }
}

bool is_node_colour_constructor(seni_node *node)
{
  if (node->type != NODE_LIST) {
    return false;
  }

  seni_node *child = safe_first(node->value.first_child);
  if (child->type != NODE_NAME) {
    return false;
  }

  // get the wlut indices of the col/* constructor functions
  //
  i32 colour_constructor_start = get_colour_constructor_start();
  i32 colour_constructor_end = get_colour_constructor_end();

  i32 native_index = child->value.i - NATIVE_START;
  if (native_index < colour_constructor_start || native_index >= colour_constructor_end) {
    return false;
  }
  
  return true;
}

seni_value_in_use get_var_value_in_use(seni_var_type type)
{
  switch(type) {
  case VAR_FLOAT:
    return USE_F;
  case VAR_LONG:
    return USE_L;
  case VAR_VECTOR:
    return USE_V;
  case VAR_COLOUR:
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
  case VAR_VECTOR: return "VAR_VECTOR";
  case VAR_COLOUR:   return "VAR_COLOUR";
  case VAR_2D:       return "VAR_2D";
  default: return "unknown seni_var type";
  }
}

void var_pretty_print(char* msg, seni_var *var)
{
  if (var == NULL) {
    SENI_ERROR("var_pretty_print: given NULL");
    return;
  }
  
  char *type = var_type_name(var);
  seni_value_in_use using = get_var_value_in_use(var->type);

  if(var->type == VAR_2D) {
    SENI_PRINT("%s: %s : [%.2f %.2f]", msg,  type, var->f32_array[0], var->f32_array[1]);
    return;
  }
    
  switch(using) {
  case USE_I:
    if (var->type == VAR_COLOUR) {
      SENI_PRINT("%s: %s : %d (%.2f, %.2f, %.2f, %.2f)", msg, type, var->value.i,
                 var->f32_array[0], var->f32_array[1], var->f32_array[2], var->f32_array[3]);
    } else {
      SENI_PRINT("%s: %s : %d", msg, type, var->value.i);
    }
    break;
  case USE_F:
    SENI_PRINT("%s: %s : %.2f", msg,  type, var->value.f);
    break;
  case USE_L:
    SENI_PRINT("%s: %s : %llu", msg, type, (long long unsigned int)(var->value.l));
    break;
  case USE_V:
    if (var->type == VAR_VECTOR) {
      SENI_PRINT("%s: %s : length %d", msg, type, vector_length(var));
    } else {
      SENI_PRINT("%s: %s", msg,  type);
    }
    break;
  default:
    SENI_ERROR("unknown using value for a seni_var: %d", using);
  }
}

bool var_serialize(seni_text_buffer *text_buffer, seni_var *var)
{
  switch(var->type) {
  case VAR_INT:
    text_buffer_sprintf(text_buffer, "INT %d", var->value.i);
    break;
  case VAR_FLOAT:
    text_buffer_sprintf(text_buffer, "FLOAT %.4f", var->value.f);
    break;
  case VAR_BOOLEAN:
    text_buffer_sprintf(text_buffer, "BOOLEAN %d", var->value.i);
    break;
  case VAR_LONG:
    text_buffer_sprintf(text_buffer, "LONG %" PRIu64 "", var->value.l);
    break;
  case VAR_NAME:
    text_buffer_sprintf(text_buffer, "NAME %d", var->value.i);
    break;
  case VAR_VECTOR:
    SENI_ERROR("var_serialize: serializing a vector?");
    return false;
    break;
  case VAR_COLOUR:
    text_buffer_sprintf(text_buffer, "COLOUR %d %.4f %.4f %.4f %.4f",
                         var->value.i,
                         var->f32_array[0],
                         var->f32_array[1],
                         var->f32_array[2],
                         var->f32_array[3]);
    break;
  case VAR_2D:
    text_buffer_sprintf(text_buffer, "2D %.4f %.4f",
                         var->f32_array[0],
                         var->f32_array[1]);
    break;
  default:
    SENI_ERROR("var_serialize: unknown seni_var type");
    return false;
  }
  
  return true;
}

bool var_deserialize(seni_var *out, seni_text_buffer *text_buffer)
{
  // assuming that the buffer is at the start of a serialized seni_var
  //
  if (text_buffer_eat_text(text_buffer, "INT")) {
    out->type = VAR_INT;
    out->value.i = text_buffer_eat_i32(text_buffer);
    return true;
  } else if (text_buffer_eat_text(text_buffer, "NAME")) {
    out->type = VAR_NAME;
    out->value.i = text_buffer_eat_i32(text_buffer);
    return true;
  } else if (text_buffer_eat_text(text_buffer, "FLOAT")) {
    out->type = VAR_FLOAT;
    out->value.f = text_buffer_eat_f32(text_buffer);
    return true;
  } else if (text_buffer_eat_text(text_buffer, "2D")) {
    out->type = VAR_2D;
    out->f32_array[0] = text_buffer_eat_f32(text_buffer);
    out->f32_array[1] = text_buffer_eat_f32(text_buffer);
    return true;
  } else if (text_buffer_eat_text(text_buffer, "COLOUR")) {
    out->type = VAR_COLOUR;
    out->value.i = text_buffer_eat_i32(text_buffer);
    out->f32_array[0] = text_buffer_eat_f32(text_buffer);
    out->f32_array[1] = text_buffer_eat_f32(text_buffer);
    out->f32_array[2] = text_buffer_eat_f32(text_buffer);
    out->f32_array[3] = text_buffer_eat_f32(text_buffer);
    return true;
  } else if (text_buffer_eat_text(text_buffer, "BOOLEAN")) {
    out->type = VAR_BOOLEAN;
    out->value.i = text_buffer_eat_i32(text_buffer);
    return true;
  } else if (text_buffer_eat_text(text_buffer, "LONG")) {
    out->type = VAR_LONG;
    out->value.l = text_buffer_eat_u64(text_buffer);
    return true;
  } else if (text_buffer_eat_text(text_buffer, "VECTOR")) {
    return false;
  }  
  
  return false;
}

void v2_as_var(seni_var *out, f32 x, f32 y)
{
  out->type = VAR_2D;
  out->f32_array[0] = x;
  out->f32_array[1] = y;
}

void f32_as_var(seni_var *out, f32 f)
{
  out->type = VAR_FLOAT;
  out->value.f = f;
}

void i32_as_var(seni_var *out, i32 i)
{
  out->type = VAR_INT;
  out->value.i = i;
}

void name_as_var(seni_var *out, i32 name)
{
  out->type = VAR_NAME;
  out->value.i = name;
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

void bytecode_pretty_print(i32 ip, seni_bytecode *b, seni_word_lut *word_lut)
{
#define PPC_BUF_SIZE 200

  char buf[PPC_BUF_SIZE];
  int buf_len = 0;
  char *buf_start = &buf[0];

#define PRINT_BC buf_len += seni_sprintf
#define BUF_ARGS buf_start + buf_len, PPC_BUF_SIZE - buf_len

  buf[0] = 0;  
  
  if (b->op == LOAD || b->op == STORE || b->op == FLU_STORE) {

    char *seg_name = memory_segment_name((seni_memory_segment_type)b->arg0.value.i);

    if (b->op == LOAD || b->op == STORE) {
      PRINT_BC(BUF_ARGS, "%d\t%s\t\t%s\t\t", ip, opcode_name(b->op), seg_name);
    } else if (b->op == FLU_STORE) {
      PRINT_BC(BUF_ARGS, "%d\t%s\t%s\t\t", ip, opcode_name(b->op), seg_name);
    } 

    seni_value_in_use using = get_var_value_in_use(b->arg1.type);
    switch(using) {
    case USE_I:
      if (b->arg1.type == VAR_COLOUR) {
        i32 type = b->arg1.value.i;
        f32 *a = b->arg1.f32_array;
        PRINT_BC(BUF_ARGS, "colour: %d (%.2f, %.2f, %.2f, %.2f)", type, a[0], a[1], a[2], a[3]);
      } else if(b->arg1.type == VAR_NAME) {
        i32 iname = b->arg1.value.i;
        if (word_lut != NULL) {
          PRINT_BC(BUF_ARGS, "name: %s (%d)", wlut_reverse_lookup(word_lut, iname), iname);
        } else {
          PRINT_BC(BUF_ARGS, "name: (%d)", iname);
        }
      } else {
        PRINT_BC(BUF_ARGS, "%d", b->arg1.value.i);
      }
      break;
    case USE_F:
      PRINT_BC(BUF_ARGS, "%.2f", b->arg1.value.f);
      break;
    case USE_L:
      PRINT_BC(BUF_ARGS, "%llu", (long long unsigned int)(b->arg1.value.l));
      break;
    case USE_V:
      if (b->arg1.type == VAR_VECTOR) {
        PRINT_BC(BUF_ARGS, "[..]len %d", vector_length(&(b->arg1)));
      } else {
        PRINT_BC(BUF_ARGS, "[..]");
      }
      break;
    default:
      PRINT_BC(BUF_ARGS, "unknown type");
    }
    
  } else if (b->op == JUMP_IF || b->op == JUMP) {
    PRINT_BC(BUF_ARGS, "%d\t%s\t\t", ip, opcode_name(b->op));
    if (b->arg0.value.i > 0) {
      PRINT_BC(BUF_ARGS, "+%d", b->arg0.value.i);
    } else if (b->arg0.value.i < 0) {
      PRINT_BC(BUF_ARGS, "%d", b->arg0.value.i);
    } else {
      PRINT_BC(BUF_ARGS, "WTF!");
    }
  } else if (b->op == NATIVE) {
    PRINT_BC(BUF_ARGS, "%d\t%s\t\t%d\t\t%d",
             ip, opcode_name(b->op), b->arg0.value.i, b->arg1.value.i);    
  } else if (b->op == PILE) {
    PRINT_BC(BUF_ARGS, "%d\t%s\t\t%d",
             ip, opcode_name(b->op), b->arg0.value.i);
  } else {
    PRINT_BC(BUF_ARGS, "%d\t%s", ip, opcode_name(b->op));
  }

  SENI_PRINT("%s", buf);
}

bool bytecode_serialize(seni_text_buffer *text_buffer, seni_bytecode *bytecode)
{
  text_buffer_sprintf(text_buffer, "%s", opcode_string[bytecode->op]);
  text_buffer_sprintf(text_buffer, " ");

  if(!var_serialize(text_buffer, &(bytecode->arg0)))
    return false;

  text_buffer_sprintf(text_buffer, " ");

  if(!var_serialize(text_buffer, &(bytecode->arg1)))
    return false;

  return true;
}

bool opcode_deserialize(seni_opcode *out, seni_text_buffer *text_buffer)
{
  if (text_buffer_eat_text(text_buffer, "LOAD"))      { *out = LOAD;      return true; }
  if (text_buffer_eat_text(text_buffer, "STORE"))     { *out = STORE;     return true; }
  if (text_buffer_eat_text(text_buffer, "SQUISH2"))   { *out = SQUISH2;   return true; }
  if (text_buffer_eat_text(text_buffer, "ADD"))       { *out = ADD;       return true; }
  if (text_buffer_eat_text(text_buffer, "SUB"))       { *out = SUB;       return true; }
  if (text_buffer_eat_text(text_buffer, "MUL"))       { *out = MUL;       return true; }
  if (text_buffer_eat_text(text_buffer, "DIV"))       { *out = DIV;       return true; }
  if (text_buffer_eat_text(text_buffer, "MOD"))       { *out = MOD;       return true; }
  if (text_buffer_eat_text(text_buffer, "NEG"))       { *out = NEG;       return true; }
  if (text_buffer_eat_text(text_buffer, "SQRT"))      { *out = SQRT;      return true; }
  if (text_buffer_eat_text(text_buffer, "EQ"))        { *out = EQ;        return true; }
  if (text_buffer_eat_text(text_buffer, "GT"))        { *out = GT;        return true; }
  if (text_buffer_eat_text(text_buffer, "LT"))        { *out = LT;        return true; }
  if (text_buffer_eat_text(text_buffer, "AND"))       { *out = AND;       return true; }
  if (text_buffer_eat_text(text_buffer, "OR"))        { *out = OR;        return true; }
  if (text_buffer_eat_text(text_buffer, "NOT"))       { *out = NOT;       return true; }
  if (text_buffer_eat_text(text_buffer, "JUMP"))      { *out = JUMP;      return true; }
  if (text_buffer_eat_text(text_buffer, "JUMP_IF"))   { *out = JUMP_IF;   return true; }
  if (text_buffer_eat_text(text_buffer, "CALL"))      { *out = CALL;      return true; }
  if (text_buffer_eat_text(text_buffer, "CALL_0"))    { *out = CALL_0;    return true; }
  if (text_buffer_eat_text(text_buffer, "RET"))       { *out = RET;       return true; }
  if (text_buffer_eat_text(text_buffer, "RET_0"))     { *out = RET_0;     return true; }
  if (text_buffer_eat_text(text_buffer, "CALL_F"))    { *out = CALL_F;    return true; }
  if (text_buffer_eat_text(text_buffer, "CALL_F_0"))  { *out = CALL_F_0;  return true; }
  if (text_buffer_eat_text(text_buffer, "NATIVE"))    { *out = NATIVE;    return true; }
  if (text_buffer_eat_text(text_buffer, "APPEND"))    { *out = APPEND;    return true; }
  if (text_buffer_eat_text(text_buffer, "PILE"))      { *out = PILE;      return true; }
  if (text_buffer_eat_text(text_buffer, "FLU_STORE")) { *out = FLU_STORE; return true; }
  if (text_buffer_eat_text(text_buffer, "MTX_LOAD"))  { *out = MTX_LOAD;  return true; }
  if (text_buffer_eat_text(text_buffer, "MTX_STORE")) { *out = MTX_STORE; return true; }
  if (text_buffer_eat_text(text_buffer, "NOP"))       { *out = NOP;       return true; }
  if (text_buffer_eat_text(text_buffer, "STOP"))      { *out = STOP;      return true; }

  return false;
}

bool bytecode_deserialize(seni_bytecode *out, seni_text_buffer *text_buffer)
{
  if (!opcode_deserialize(&(out->op), text_buffer))
    return false;
  
  text_buffer_eat_space(text_buffer);
  
  if(!var_deserialize(&(out->arg0), text_buffer))
    return false;
  
  text_buffer_eat_space(text_buffer);
  
  if(!var_deserialize(&(out->arg1), text_buffer))
    return false;
  
  return true;
}

seni_env *env_allocate()
{
  seni_env *e = (seni_env *)calloc(1, sizeof(seni_env));
  e->word_lut = wlut_allocate();

  declare_bindings(e->word_lut, e);
  
  return e;
}

void env_free(seni_env *e)
{
  wlut_free(e->word_lut);
  free(e);
}

// **************************************************
// Program
// **************************************************

char *opcode_name(seni_opcode opcode)
{
#define STR(x) #x
  
  switch(opcode) {
#define OPCODE(id,_) case id: return STR(id);
#include "seni_opcodes.h"
#undef OPCODE
  default:
    return "unknown opcode";
  }
#undef STR
}

seni_program *program_allocate(i32 code_max_size)
{
  seni_program *program = (seni_program *)calloc(1, sizeof(seni_program));

  if (code_max_size > 0) {
    program->code = (seni_bytecode *)calloc(code_max_size, sizeof(seni_bytecode));
  }
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

i32 program_stop_location(seni_program *program)
{
  // the final opcode in the program will always be a STOP
  return program->code_size - 1;
}

void program_pretty_print(seni_program *program)
{
  for (i32 i = 0; i < program->code_size; i++) {
    seni_bytecode *b = &(program->code[i]);
    bytecode_pretty_print(i, b, program->word_lut);
  }
  SENI_PRINT("\n");
}

// note: only using the serialize/deserialize functions for code in traits
// so the program->fn_info isn't being serialized. (it should if there's
// ever a need for proper program serialization)
//
bool program_serialize(seni_text_buffer *text_buffer, seni_program *program)
{
  text_buffer_sprintf(text_buffer, "%d", program->code_max_size);
  text_buffer_sprintf(text_buffer, " ");
  text_buffer_sprintf(text_buffer, "%d", program->code_size);
  text_buffer_sprintf(text_buffer, " ");

  seni_bytecode *bytecode = program->code;
  for (i32 i = 0; i < program->code_size; i++) {
    if (!bytecode_serialize(text_buffer, bytecode)) {
      SENI_ERROR("program_serialize: bytecode_serialize at instruction %d", i);
      return false;
    }
    // interleave bytecodes with a space
    if (i < program->code_size - 1) {
      text_buffer_sprintf(text_buffer, " ");
    }
    bytecode++;
  }
  
  return true;
}

bool program_deserialize(seni_program *out, seni_text_buffer *text_buffer)
{
  out->code_max_size = text_buffer_eat_i32(text_buffer);
  text_buffer_eat_space(text_buffer);
  out->code_size = text_buffer_eat_i32(text_buffer);
  text_buffer_eat_space(text_buffer);

  if(out->code != NULL) {
    free(out->code);
  }
  out->code = (seni_bytecode *)calloc(out->code_max_size, sizeof(seni_bytecode));

  for (i32 i = 0; i < out->code_size; i++) {
    if(!bytecode_deserialize(&(out->code[i]), text_buffer)) {
      return false;
    }
    if (i < out->code_size - 1) {    
      text_buffer_eat_space(text_buffer);
    }
  }
  
  return true;
}



// **************************************************
// Virtual Machine
// **************************************************

seni_vm *vm_allocate(i32 stack_size, i32 heap_size, i32 heap_min_size, i32 vertex_packet_num_vertices)
{
  seni_vm *vm = (seni_vm *)calloc(1, sizeof(seni_vm));

  vm->render_data = NULL;

  vm->stack_size = stack_size;
  vm->stack = (seni_var *)calloc(stack_size, sizeof(seni_var));

  vm->heap_size = heap_size;
  vm->heap_slab = (seni_var *)calloc(heap_size, sizeof(seni_var));

  vm->heap_avail_size_before_gc = heap_min_size;

  vm->matrix_stack = matrix_stack_allocate();

  // prepare storage for vertices
  seni_render_data *render_data = render_data_allocate(vertex_packet_num_vertices);
  vm->render_data = render_data;

  vm->prng_state = (seni_prng_state *)calloc(1, sizeof(seni_prng_state));
  
  vm_reset(vm);

  return vm;
}

void vm_reset(seni_vm *vm)
{
  seni_var *var;
  i32 base_offset = 0;
  i32 i;

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

  vm->heap_avail = NULL;
  for (i = 0; i < vm->heap_size; i++) {
    vm->heap_slab[i].next = NULL;
    vm->heap_slab[i].prev = NULL;
  }

  var = vm->heap_slab;
  for (i = 0; i < vm->heap_size; i++) {
    var[i].mark = false;
    DL_APPEND(vm->heap_avail, &(var[i]));
  }

  vm->heap_avail_size = vm->heap_size;

  matrix_stack_reset(vm->matrix_stack);

  render_data_free_render_packets(vm->render_data);
  add_render_packet(vm->render_data);
}


void vm_free_render_data(seni_vm *vm)
{
  render_data_free(vm->render_data);
  vm->render_data = NULL;
}

void vm_free(seni_vm *vm)
{
  vm_free_render_data(vm);
  matrix_stack_free(vm->matrix_stack);
  free(vm->stack);
  free(vm->heap_slab);
  free(vm->prng_state);
  free(vm);
}

void vm_pretty_print(seni_vm *vm, char* msg)
{
  SENI_LOG("%s\tvm: fp:%d sp:%d ip:%d local:%d",
             msg,
             vm->fp,
             vm->sp,
             vm->ip,
             vm->local);

  seni_var *fp = &(vm->stack[vm->fp]);
  i32 onStackFP = (fp + 0)->value.i;
  i32 onStackIP = (fp + 1)->value.i;
  i32 onStackNumArgs = (fp + 2)->value.i;
  SENI_LOG("\ton stack: fp:%d ip:%d numArgs:%d", onStackFP, onStackIP, onStackNumArgs);
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

// [ ] <<- this is the VAR_VECTOR (value.v points to the first heap allocated seni_var)
//  |
//  v 
// [4] -> [7] -> [3] -> [5] -> NULL  <<- these are heap allocated seni_vars
//
void vector_construct(seni_var *head)
{
  // assuming that it's ok to wipe out head->value.v
  head->type = VAR_VECTOR;
  head->value.v = NULL;           // attach vec_rc to vec_head
}

i32 vector_length(seni_var *var)
{
  if (var->type != VAR_VECTOR) {
    return 0;
  }

  i32 len = 0;
  seni_var *v = var->value.v;

  while (v != NULL) {
    len++;
    v = v->next;
  }

  return len;
}

seni_var *vector_get(seni_var *var, i32 index)
{
  if (var->type != VAR_VECTOR) {
    return 0;
  }

  seni_var *res = var->value.v;

  while (index > 0) {
    index--;
    res = res->next;
    if (res == NULL) {
      // index is greater than the length of the vector
      return NULL;
    }
  }

  return res;
}

void vector_append_heap_var(seni_var *head, seni_var *val)
{
  // assuming that head is VAR_VECTOR and val is a seni_var from the heap
  DL_APPEND(head->value.v, val);
}

seni_var *vector_append_i32(seni_vm *vm, seni_var *head, i32 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("vector_append_i32");
    return NULL;
  }
  
  v->type = VAR_INT;
  v->value.i = val;

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *vector_append_f32(seni_vm *vm, seni_var *head, f32 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("vector_append_f32");
    return NULL;
  }
  
  v->type = VAR_FLOAT;
  v->value.f = val;

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *vector_append_u64(seni_vm *vm, seni_var *head, u64 val)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("vector_append_u64");
    return NULL;
  }
  v->type = VAR_LONG;
  v->value.l = val;

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *vector_append_col(seni_vm *vm, seni_var *head, seni_colour *col)
{
  seni_var *v = var_get_from_heap(vm);
  if (v == NULL) {
    SENI_ERROR("vector_append_col");
    return NULL;
  }

  colour_as_var(v, col);

  DL_APPEND(head->value.v, v);

  return v;
}

seni_var *var_get_from_heap(seni_vm *vm)
{
  seni_var *head = vm->heap_avail;

  if (head != NULL) {
    DL_DELETE(vm->heap_avail, head);
  } else {
    SENI_ERROR("out of heap memory error");
    return NULL;
  }

  vm->heap_avail_size--;

  head->next = NULL;
  head->prev = NULL;

  head->value.i = 0;
  head->type = VAR_INT;         // just make sure that it isn't VAR_VECTOR from a previous allocation

  return head;
}

void var_copy(seni_var *dest, seni_var *src)
{
  if (dest == src) {
    return;
  }

  dest->type = src->type;
  dest->f32_array[0] = src->f32_array[0];
  dest->f32_array[1] = src->f32_array[1];
  dest->f32_array[2] = src->f32_array[2];
  dest->f32_array[3] = src->f32_array[3];

  seni_value_in_use using = get_var_value_in_use(src->type);
  
  if (using == USE_I) {
    dest->value.i = src->value.i;
  } else if (using == USE_F) {
    dest->value.f = src->value.f;
  } else if (using == USE_L) {
    dest->value.l = src->value.l;
  } else if (using == USE_V) {
    if (src->type == VAR_VECTOR) {
      dest->value.v = src->value.v;
    } else {
      SENI_ERROR("what the fuck?\n");
    }
  } else {
    SENI_ERROR("unknown seni_value_in_use for var_copy");
  }
}

#ifdef SENI_DEBUG_MODE

void vm_debug_info_reset(seni_vm *vm)
{
  vm->opcodes_executed = 0;
}

void vm_debug_info_print(seni_vm *vm)
{
  SENI_PRINT("*** vm_debug_info_print ***");
  SENI_PRINT("bytecodes executed:\t%llu", (long long unsigned int)(vm->opcodes_executed));
  SENI_PRINT("bytecode execution time:\t%.2f msec", vm->execution_time);
}

#endif

void lang_subsystem_startup()
{
  // start with 1 slab of 100, allocate upto a max of 10 slabs
  g_var_pool = var_pool_allocate(1, 100, 50);  
}

void lang_subsystem_shutdown()
{
  var_pool_free(g_var_pool);
}

seni_var *var_get_from_pool()
{
  seni_var *var = var_pool_get(g_var_pool);

  // reset the value field to NULL, required if this var is going to be used as a VAR_VECTOR
  var->value.v = NULL;

  return var;
}

void var_return_to_pool(seni_var *var)
{
  var_pool_return(g_var_pool, var);
}

