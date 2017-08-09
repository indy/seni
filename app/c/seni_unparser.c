#include "seni_unparser.h"
#include "seni_lang.h"
#include "seni_ga.h"
#include "seni_printf.h"

#include "string.h"
#include <stdlib.h>

#include <string.h>
#ifdef SENI_BUILD_WASM
#include <webassembly.h>
#else
#include <stdio.h>
#endif
#include "stdarg.h"

typedef struct {
  char *buffer;
  i32 buffer_size;

  char *cursor;
  i32 current_size;
  
} seni_buffer_writer;

void buffer_writer_sprintf(seni_buffer_writer *buffer_writer, char const * fmt, ... )
{
  va_list va;
  va_start(va, fmt);
  int len = seni_vsprintf(buffer_writer->cursor, buffer_writer->current_size, fmt, va);
  va_end(va);

  buffer_writer->current_size -= len;
  if (buffer_writer->current_size > 0) {
    buffer_writer->cursor += len;
  } else {
    SENI_ERROR("seni_buffer_writer: buffer is full");
  }

}

i32 count_decimals(seni_node *float_node)
{
  if (float_node->type != NODE_FLOAT) {
    return 0;
  }

  i32 i = 0;
  for (i = 0; i < float_node->src_len; i++) {
    if (float_node->src[i] == '.') {
      i++;
      break;
    } 
  }

  i32 res = float_node->src_len - i;

  return res;
}

void format_node_value_float(seni_buffer_writer *buffer_writer, seni_node *node)
{
  i32 decimals = count_decimals(node);

  switch(decimals) {
  case 0: buffer_writer_sprintf(buffer_writer, "%.0f", node->value.f); break;
  case 1: buffer_writer_sprintf(buffer_writer, "%.1f", node->value.f); break;
  case 2: buffer_writer_sprintf(buffer_writer, "%.2f", node->value.f); break;
  case 3: buffer_writer_sprintf(buffer_writer, "%.3f", node->value.f); break;
  case 4: buffer_writer_sprintf(buffer_writer, "%.4f", node->value.f); break;
  case 5: buffer_writer_sprintf(buffer_writer, "%.5f", node->value.f); break;
  case 6: buffer_writer_sprintf(buffer_writer, "%.6f", node->value.f); break;
  case 7: buffer_writer_sprintf(buffer_writer, "%.7f", node->value.f); break;
  case 8: buffer_writer_sprintf(buffer_writer, "%.8f", node->value.f); break;
  case 9: buffer_writer_sprintf(buffer_writer, "%.9f", node->value.f); break;
  default: buffer_writer_sprintf(buffer_writer, "%f", node->value.f);
  };
}

void format_node_value(seni_buffer_writer *buffer_writer, seni_env *env, char *value, seni_node *node)
{
  char *c;
  
  switch (node->type) {

  case NODE_LIST:
    SENI_ERROR("NODE_LIST ???");
    break;
  case NODE_VECTOR:
    SENI_ERROR("NODE_VECTOR ???");
    break;
  case NODE_INT:
    buffer_writer_sprintf(buffer_writer, "%d", node->value.i);
    break;
  case NODE_FLOAT:
    format_node_value_float(buffer_writer, node);
    break;
  case NODE_NAME:
    c = wlut_reverse_lookup(env->wl, node->value.i);
    buffer_writer_sprintf(buffer_writer, "%s", c);
    break;
  case NODE_LABEL:
    c = wlut_reverse_lookup(env->wl, node->value.i);
    buffer_writer_sprintf(buffer_writer, "%s:", c);
    break;
  case NODE_STRING:
    c = wlut_reverse_lookup(env->wl, node->value.i);
    buffer_writer_sprintf(buffer_writer, "\"%s\"", c);
    break;
  case NODE_WHITESPACE:
    buffer_writer_sprintf(buffer_writer, "%s", node->value.s);
    break;
  case NODE_COMMENT:
    buffer_writer_sprintf(buffer_writer, "%s", node->value.s);
    break;
  default:
    SENI_ERROR("???");
    /*
  case NODE_BOOLEAN:
    string_append(out, value === "#t" ? "true" : "false")
    break;
    */
  };
}

seni_node *unparse_ast_node(seni_buffer_writer *buffer_writer, seni_env *env, seni_node *ast, seni_genotype *genotype)
{
  seni_node *n;
  
  if (ast->alterable == true) {

    buffer_writer_sprintf(buffer_writer, "{");
    if (ast->parameter_prefix != NULL) {
      unparse_ast_node(buffer_writer, env, ast->parameter_prefix, genotype);
    }

    // // use value from genotype
    // if (node.type === NodeType.VECTOR) {
    //   // a vector requires multiple values from the genotype
    //   [v, geno] = getMultipleValuesFromGenotype(node.children, geno);
    // } else {
    //   [v, geno] = pullValueFromGenotype(geno);
    //   v = formatNodeValue(v, node);
    // }
    format_node_value(buffer_writer, env, NULL, ast);    

    n = ast->parameter_ast;
    while (n != NULL) {
      unparse_ast_node(buffer_writer, env, n, genotype);
      n = n->next;
    }

    buffer_writer_sprintf(buffer_writer, "}");    
  } else {
    if (ast->type == NODE_LIST) {

      // if (node.usingAbbreviation) {
      // }
      // else
      {
        buffer_writer_sprintf(buffer_writer, "(");

        n = ast->value.first_child;
        while (n != NULL) {
          unparse_ast_node(buffer_writer, env, n, genotype);
          n = n->next;
        }
        buffer_writer_sprintf(buffer_writer, ")");
        
      }
      
    } else if (ast->type == NODE_VECTOR) {
      {
        buffer_writer_sprintf(buffer_writer, "[");

        n = ast->value.first_child;
        while (n != NULL) {
          unparse_ast_node(buffer_writer, env, n, genotype);
          n = n->next;
        }
        buffer_writer_sprintf(buffer_writer, "]");
        
      }
 
    } else {
      format_node_value(buffer_writer, env, NULL, ast);
    }

    // term = formatNodeValue(v, node);
  }
  
  return ast->next;
}

// out is a pre-allocated array
//
bool unparse(char *out, i32 out_size, seni_env *env, seni_node *ast, seni_genotype *genotype)
{

  seni_buffer_writer buffer_writer = {
    .buffer = out,
    .buffer_size = out_size,
    .cursor = out,
    .current_size = out_size
  };
  
  seni_node *n = ast;
  while (n != NULL) {
    n = unparse_ast_node(&buffer_writer, env, n, genotype);
  }

  return true;
}
