#include "seni_unparser.h"

#include "seni_ga.h"
#include "seni_lang.h"
#include "seni_printf.h"
#include "seni_text_buffer.h"

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

void print_decimals(seni_text_buffer *text_buffer, i32 decimals, f32 f)
{
  switch(decimals) {
  case 0: text_buffer_sprintf(text_buffer, "%.0f", f); break;
  case 1: text_buffer_sprintf(text_buffer, "%.1f", f); break;
  case 2: text_buffer_sprintf(text_buffer, "%.2f", f); break;
  case 3: text_buffer_sprintf(text_buffer, "%.3f", f); break;
  case 4: text_buffer_sprintf(text_buffer, "%.4f", f); break;
  case 5: text_buffer_sprintf(text_buffer, "%.5f", f); break;
  case 6: text_buffer_sprintf(text_buffer, "%.6f", f); break;
  case 7: text_buffer_sprintf(text_buffer, "%.7f", f); break;
  case 8: text_buffer_sprintf(text_buffer, "%.8f", f); break;
  case 9: text_buffer_sprintf(text_buffer, "%.9f", f); break;
  default: text_buffer_sprintf(text_buffer, "%f", f);
  };
}

void format_node_value_float(seni_text_buffer *text_buffer, seni_node *node)
{
  i32 decimals = count_decimals(node);
  print_decimals(text_buffer, decimals, node->value.f);
}

void format_var_value_float(seni_text_buffer *text_buffer, seni_node *node, seni_var *var)
{
  i32 decimals = count_decimals(node);
  print_decimals(text_buffer, decimals, var->value.f);
}

void format_node_value(seni_text_buffer *text_buffer, seni_word_lut *word_lut, char *value, seni_node *node)
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
    format_node_value_float(text_buffer, node);
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

seni_gene *genotype_pull_gene(seni_genotype *genotype)
{
  seni_gene *gene = genotype->current_gene;
  if (gene == NULL) {
    SENI_ERROR("genotype_pull_gene: current gene is null");
    return NULL;
  }

  genotype->current_gene = genotype->current_gene->next;

  return gene;
}

void format_var_value(seni_text_buffer *text_buffer, seni_node *node, seni_genotype *genotype)
{
  seni_gene *gene = genotype_pull_gene(genotype); 
  seni_var *var = gene->var;
  
  switch (var->type) {
  case VAR_INT:
    text_buffer_sprintf(text_buffer, "%d", var->value.i);
    break;
  case VAR_FLOAT:
    format_var_value_float(text_buffer, node, var);
    break;
  case VAR_VECTOR:
    // a vector requires multiple values from the genotype
    SENI_ERROR("???");
    break;
  case VAR_COLOUR:
    SENI_ERROR("???");
    break;
  case VAR_2D:
    SENI_ERROR("???");
    break;
  default:
    SENI_ERROR("???");
  };
}

seni_node *unparse_ast_node(seni_text_buffer *text_buffer, seni_word_lut *word_lut, seni_node *ast, seni_genotype *genotype)
{
  seni_node *n;
  
  if (ast->alterable) {

    text_buffer_sprintf(text_buffer, "{");
    if (ast->parameter_prefix != NULL) {
      unparse_ast_node(text_buffer, word_lut, ast->parameter_prefix, genotype);
    }

    // // use value from genotype
    // if (node.type === NodeType.VECTOR) {
    //   // a vector requires multiple values from the genotype
    //   [v, geno] = getMultipleValuesFromGenotype(node.children, geno);
    // } else {
    //   [v, geno] = pullValueFromGenotype(geno);
    //   v = formatNodeValue(v, node);
    // }
    format_var_value(text_buffer, ast, genotype);
    
    n = ast->parameter_ast;
    while (n != NULL) {
      unparse_ast_node(text_buffer, word_lut, n, genotype);
      n = n->next;
    }

    text_buffer_sprintf(text_buffer, "}");    
  } else {
    if (ast->type == NODE_LIST) {

      // if (node.usingAbbreviation) {
      // }
      // else
      {
        text_buffer_sprintf(text_buffer, "(");

        n = ast->value.first_child;
        while (n != NULL) {
          unparse_ast_node(text_buffer, word_lut, n, genotype);
          n = n->next;
        }
        text_buffer_sprintf(text_buffer, ")");
        
      }
      
    } else if (ast->type == NODE_VECTOR) {
      {
        text_buffer_sprintf(text_buffer, "[");

        n = ast->value.first_child;
        while (n != NULL) {
          unparse_ast_node(text_buffer, word_lut, n, genotype);
          n = n->next;
        }
        text_buffer_sprintf(text_buffer, "]");
        
      }
 
    } else {
      format_node_value(text_buffer, word_lut, NULL, ast);
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

  return true;
}
