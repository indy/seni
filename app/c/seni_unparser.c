#include "seni_unparser.h"

#include "seni_ga.h"
#include "seni_lang.h"
#include "seni_printf.h"
#include "seni_text_buffer.h"
#include "seni_keyword_iname.h"
#include "seni_colour.h"

#include "string.h"
#include <stdlib.h>

#include <string.h>
#ifdef SENI_BUILD_WASM
#include <webassembly.h>
#else
#include <stdio.h>
#endif

i32 count_decimals(seni_node *float_node)
{
  if (float_node->type != NODE_FLOAT) {
    return 0;
  }

  // SENI_LOG("src = %s", float_node->src);
  // SENI_LOG("src_len = %d", float_node->src_len);
  
  i32 i = 0;
  for (i = 0; i < float_node->src_len; i++) {
    if (float_node->src[i] == '.') {
      i++;
      break;
    } 
  }

  i32 res = float_node->src_len - i;

  // SENI_LOG("i = %d", i);
  // SENI_LOG("res = %d", res);

  return res;
}

void format_float_using_node(seni_text_buffer *text_buffer, seni_node *node, f32 f)
{
  i32 decimals = count_decimals(node);
  text_buffer_sprintf(text_buffer, "%.*f", decimals, f);
}

void format_var_value_colour(seni_text_buffer *text_buffer, seni_node *node, seni_var *var)
{
  node = NULL;
  switch(var->value.i) {
  case RGB:
    text_buffer_sprintf(text_buffer, "(col/rgb r: %.2f g: %.2f b: %.2f alpha: %.2f)",
                        var->f32_array[0], var->f32_array[1], var->f32_array[2], var->f32_array[3]);
    break;
  case HSL:
    text_buffer_sprintf(text_buffer, "(col/hsl h: %.2f s: %.2f l: %.2f alpha: %.2f)",
                        var->f32_array[0], var->f32_array[1], var->f32_array[2], var->f32_array[3]);
    break;
  case LAB:
    text_buffer_sprintf(text_buffer, "(col/lab l: %.2f a: %.2f b: %.2f alpha: %.2f)",
                        var->f32_array[0], var->f32_array[1], var->f32_array[2], var->f32_array[3]);
    break;
  case HSV:
    text_buffer_sprintf(text_buffer, "(col/hsv h: %.2f s: %.2f v: %.2f alpha: %.2f)",
                        var->f32_array[0], var->f32_array[1], var->f32_array[2], var->f32_array[3]);
    break;
  }
}

void format_node_value(seni_text_buffer *text_buffer, seni_word_lut *word_lut, seni_node *node)
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
    text_buffer_sprintf(text_buffer, "%d", node->value.i);
    break;
  case NODE_FLOAT:
    format_float_using_node(text_buffer, node, node->value.f);
    break;
  case NODE_NAME:
    c = wlut_reverse_lookup(word_lut, node->value.i);
    text_buffer_sprintf(text_buffer, "%s", c);
    break;
  case NODE_LABEL:
    c = wlut_reverse_lookup(word_lut, node->value.i);
    text_buffer_sprintf(text_buffer, "%s:", c);
    break;
  case NODE_STRING:
    c = wlut_reverse_lookup(word_lut, node->value.i);
    text_buffer_sprintf(text_buffer, "\"%s\"", c);
    break;
  case NODE_WHITESPACE:
    text_buffer_sprintf(text_buffer, "%s", node->value.s);
    break;
  case NODE_COMMENT:
    text_buffer_sprintf(text_buffer, "%s", node->value.s);
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

void format_var_value(seni_text_buffer *text_buffer, seni_node *node, seni_genotype *genotype, seni_word_lut *word_lut)
{
  seni_gene *gene = genotype_pull_gene(genotype); 
  seni_var *var = gene->var;
  char *name = NULL;            // used by VAR_NAME
  seni_node *n;

  switch (var->type) {
  case VAR_INT:
    text_buffer_sprintf(text_buffer, "%d", var->value.i);
    break;
  case VAR_FLOAT:
    format_float_using_node(text_buffer, node, var->value.f);
    break;
  case VAR_NAME:
    name = wlut_reverse_lookup(word_lut, var->value.i);
    text_buffer_sprintf(text_buffer, "%s", name);
    break;
  case VAR_VECTOR:
    SENI_ERROR("vector ???");

    break;
  case VAR_COLOUR:
    format_var_value_colour(text_buffer, node, var);
    break;
  case VAR_2D:

    // node is a NODE_VECTOR
    
    text_buffer_sprintf(text_buffer, "[");
    
    n = node->value.first_child;
    while (n && n->type != NODE_FLOAT) {
      format_node_value(text_buffer, word_lut, n);
      n = n->next;
    }
    if (n == NULL) {
      SENI_ERROR("null node trying to unparse first float in VAR_2D");
      return;
    }

    format_float_using_node(text_buffer, n, var->f32_array[0]);
    n = n->next;

    while (n && n->type != NODE_FLOAT) {
      format_node_value(text_buffer, word_lut, n);
      n = n->next;
    }
    if (n == NULL) {
      SENI_ERROR("null node trying to unparse second float in VAR_2D");
      return;
    }

    format_float_using_node(text_buffer, n, var->f32_array[1]);
    n = n->next;

    // output any trailing spaces
    while (n) {
      format_node_value(text_buffer, word_lut, n);
      n = n->next;
    }

    text_buffer_sprintf(text_buffer, "]");
    break;
  default:
    SENI_ERROR("???");
  };
}

void unparse_alterable_vector(seni_text_buffer *text_buffer, seni_word_lut *word_lut, seni_node *ast, seni_genotype *genotype)
{
  text_buffer_sprintf(text_buffer, "[");

  seni_node *n = ast->value.first_child;

  while (n) {
    if (n->type == NODE_WHITESPACE || n->type == NODE_COMMENT) {
      format_node_value(text_buffer, word_lut, n);
    } else {
      format_var_value(text_buffer, n, genotype, word_lut);
    }
    n = n->next;
  }

  text_buffer_sprintf(text_buffer, "]");
}
  
seni_node *unparse_ast_node(seni_text_buffer *text_buffer, seni_word_lut *word_lut, seni_node *ast, seni_genotype *genotype)
{
  seni_node *n;
  
  if (ast->alterable) {

    text_buffer_sprintf(text_buffer, "{");
    if (ast->parameter_prefix != NULL) {
      unparse_ast_node(text_buffer, word_lut, ast->parameter_prefix, genotype);
    }

    if (ast->type == NODE_VECTOR) {
      unparse_alterable_vector(text_buffer, word_lut, ast, genotype);
    } else {
      format_var_value(text_buffer, ast, genotype, word_lut);
    }

    n = ast->parameter_ast;
    while (n != NULL) {
      unparse_ast_node(text_buffer, word_lut, n, genotype);
      n = n->next;
    }

    text_buffer_sprintf(text_buffer, "}");    

  } else {
    
    if (ast->type == NODE_LIST) {
      n = safe_first(ast->value.first_child);
      if (n->type == NODE_NAME && n->value.i == INAME_QUOTE) {
        // rather than outputing: (quote (1 2 3))
        // we want: '(1 2 3)
        //
        text_buffer_sprintf(text_buffer, "'");

        n = n->next;       // skip past the "quote"
        n = safe_next(n);  // skip past the whitespace
        
        while (n != NULL) {
          unparse_ast_node(text_buffer, word_lut, n, genotype);
          n = n->next;
        }
        
      } else {
        text_buffer_sprintf(text_buffer, "(");

        while (n != NULL) {
          unparse_ast_node(text_buffer, word_lut, n, genotype);
          n = n->next;
        }
        text_buffer_sprintf(text_buffer, ")");
        
      }
      
    } else if (ast->type == NODE_VECTOR) {
      n = safe_first(ast->value.first_child);
      text_buffer_sprintf(text_buffer, "[");

      while (n != NULL) {
        unparse_ast_node(text_buffer, word_lut, n, genotype);
        n = n->next;
      }
      text_buffer_sprintf(text_buffer, "]");
 
    } else {
      format_node_value(text_buffer, word_lut, ast);
    }

    // term = formatNodeValue(v, node);
  }
  
  return ast->next;
}

// out is a pre-allocated array
//
bool unparse(seni_text_buffer *text_buffer, seni_word_lut *word_lut, seni_node *ast, seni_genotype *genotype)
{
  seni_node *n = ast;
  genotype->current_gene = genotype->genes;
  while (n != NULL) {
    n = unparse_ast_node(text_buffer, word_lut, n, genotype);
  }

  if (genotype->current_gene && genotype->current_gene->next != NULL) {
    SENI_ERROR("unparse: genes remaining after unparse?");
    return false;
  }

  return true;
}
