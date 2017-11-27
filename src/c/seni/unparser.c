#include "unparser.h"

#include "colour.h"
#include "cursor.h"
#include "genetic.h"
#include "keyword_iname.h"
#include "lang.h"
#include "printf.h"

#include "string.h"
#include <stdlib.h>

#include <string.h>
#ifdef SENI_BUILD_WASM
#include <webassembly.h>
#else
#include <stdio.h>
#endif

i32 count_decimals(seni_node *float_node) {
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

void format_float_using_node(seni_cursor *cursor, seni_node *node, f32 f) {
  i32 decimals = count_decimals(node);
  cursor_sprintf(cursor, "%.*f", decimals, f);
}

void format_var_value_colour(seni_cursor *cursor, seni_node *node, seni_var *var) {
  node = NULL;
  switch (var->value.i) {
  case RGB:
    cursor_sprintf(cursor,
                   "(col/rgb r: %.2f g: %.2f b: %.2f alpha: %.2f)",
                   var->f32_array[0],
                   var->f32_array[1],
                   var->f32_array[2],
                   var->f32_array[3]);
    break;
  case HSL:
    cursor_sprintf(cursor,
                   "(col/hsl h: %.2f s: %.2f l: %.2f alpha: %.2f)",
                   var->f32_array[0],
                   var->f32_array[1],
                   var->f32_array[2],
                   var->f32_array[3]);
    break;
  case LAB:
    cursor_sprintf(cursor,
                   "(col/lab l: %.2f a: %.2f b: %.2f alpha: %.2f)",
                   var->f32_array[0],
                   var->f32_array[1],
                   var->f32_array[2],
                   var->f32_array[3]);
    break;
  case HSV:
    cursor_sprintf(cursor,
                   "(col/hsv h: %.2f s: %.2f v: %.2f alpha: %.2f)",
                   var->f32_array[0],
                   var->f32_array[1],
                   var->f32_array[2],
                   var->f32_array[3]);
    break;
  }
}

void format_node_value(seni_cursor *cursor, seni_word_lut *word_lut, seni_node *node) {
  char *c;

  switch (node->type) {

  case NODE_LIST:
    SENI_ERROR("NODE_LIST ???");
    break;
  case NODE_VECTOR:
    SENI_ERROR("NODE_VECTOR ???");
    break;
  case NODE_INT:
    cursor_sprintf(cursor, "%d", node->value.i);
    break;
  case NODE_FLOAT:
    format_float_using_node(cursor, node, node->value.f);
    break;
  case NODE_NAME:
    c = wlut_reverse_lookup(word_lut, node->value.i);
    cursor_sprintf(cursor, "%s", c);
    break;
  case NODE_LABEL:
    c = wlut_reverse_lookup(word_lut, node->value.i);
    cursor_sprintf(cursor, "%s:", c);
    break;
  case NODE_STRING:
    c = wlut_reverse_lookup(word_lut, node->value.i);
    cursor_sprintf(cursor, "\"%s\"", c);
    break;
  case NODE_WHITESPACE:
    cursor_strncpy(cursor, node->src, node->src_len);
    break;
  case NODE_COMMENT:
    cursor_strncpy(cursor, node->src, node->src_len);
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

void format_var_value(seni_cursor *  cursor,
                      seni_node *    node,
                      seni_genotype *genotype,
                      seni_word_lut *word_lut) {
  seni_gene *gene = genotype_pull_gene(genotype);
  seni_var * var  = gene->var;
  char *     name = NULL; // used by VAR_NAME
  seni_node *n;

  switch (var->type) {
  case VAR_INT:
    cursor_sprintf(cursor, "%d", var->value.i);
    break;
  case VAR_FLOAT:
    format_float_using_node(cursor, node, var->value.f);
    break;
  case VAR_NAME:
    name = wlut_reverse_lookup(word_lut, var->value.i);
    cursor_sprintf(cursor, "%s", name);
    break;
  case VAR_VECTOR:
    SENI_ERROR("vector ???");

    break;
  case VAR_COLOUR:
    format_var_value_colour(cursor, node, var);
    break;
  case VAR_2D:

    // node is a NODE_VECTOR

    cursor_sprintf(cursor, "[");

    n = node->value.first_child;
    while (n && n->type != NODE_FLOAT) {
      format_node_value(cursor, word_lut, n);
      n = n->next;
    }
    if (n == NULL) {
      SENI_ERROR("null node trying to unparse first float in VAR_2D");
      return;
    }

    format_float_using_node(cursor, n, var->f32_array[0]);
    n = n->next;

    while (n && n->type != NODE_FLOAT) {
      format_node_value(cursor, word_lut, n);
      n = n->next;
    }
    if (n == NULL) {
      SENI_ERROR("null node trying to unparse second float in VAR_2D");
      return;
    }

    format_float_using_node(cursor, n, var->f32_array[1]);
    n = n->next;

    // output any trailing spaces
    while (n) {
      format_node_value(cursor, word_lut, n);
      n = n->next;
    }

    cursor_sprintf(cursor, "]");
    break;
  default:
    SENI_ERROR("???");
  };
}

void unparse_alterable_vector(seni_cursor *  cursor,
                              seni_word_lut *word_lut,
                              seni_node *    ast,
                              seni_genotype *genotype) {
  cursor_sprintf(cursor, "[");

  seni_node *n = ast->value.first_child;

  while (n) {
    if (n->type == NODE_WHITESPACE || n->type == NODE_COMMENT) {
      format_node_value(cursor, word_lut, n);
    } else {
      format_var_value(cursor, n, genotype, word_lut);
    }
    n = n->next;
  }

  cursor_sprintf(cursor, "]");
}

seni_node *unparse_ast_node(seni_cursor *  cursor,
                            seni_word_lut *word_lut,
                            seni_node *    ast,
                            seni_genotype *genotype) {
  seni_node *n;

  if (ast->alterable) {

    cursor_sprintf(cursor, "{");
    if (ast->parameter_prefix != NULL) {
      unparse_ast_node(cursor, word_lut, ast->parameter_prefix, genotype);
    }

    if (ast->type == NODE_VECTOR) {
      unparse_alterable_vector(cursor, word_lut, ast, genotype);
    } else {
      format_var_value(cursor, ast, genotype, word_lut);
    }

    n = ast->parameter_ast;
    while (n != NULL) {
      unparse_ast_node(cursor, word_lut, n, genotype);
      n = n->next;
    }

    cursor_sprintf(cursor, "}");

  } else {

    if (ast->type == NODE_LIST) {
      n = safe_first(ast->value.first_child);
      if (n->type == NODE_NAME && n->value.i == INAME_QUOTE) {
        // rather than outputing: (quote (1 2 3))
        // we want: '(1 2 3)
        //
        cursor_sprintf(cursor, "'");

        n = n->next;      // skip past the "quote"
        n = safe_next(n); // skip past the whitespace

        while (n != NULL) {
          unparse_ast_node(cursor, word_lut, n, genotype);
          n = n->next;
        }

      } else {
        cursor_sprintf(cursor, "(");

        while (n != NULL) {
          unparse_ast_node(cursor, word_lut, n, genotype);
          n = n->next;
        }
        cursor_sprintf(cursor, ")");
      }

    } else if (ast->type == NODE_VECTOR) {
      n = safe_first(ast->value.first_child);
      cursor_sprintf(cursor, "[");

      while (n != NULL) {
        unparse_ast_node(cursor, word_lut, n, genotype);
        n = n->next;
      }
      cursor_sprintf(cursor, "]");

    } else {
      format_node_value(cursor, word_lut, ast);
    }

    // term = formatNodeValue(v, node);
  }

  return ast->next;
}

// out is a pre-allocated array
//
bool unparse(seni_cursor *  cursor,
             seni_word_lut *word_lut,
             seni_node *    ast,
             seni_genotype *genotype) {
  seni_node *n           = ast;
  genotype->current_gene = genotype->genes;
  while (n != NULL) {
    n = unparse_ast_node(cursor, word_lut, n, genotype);
  }

  if (genotype->current_gene && genotype->current_gene->next != NULL) {
    SENI_ERROR("unparse: genes remaining after unparse?");
    return false;
  }

  return true;
}
