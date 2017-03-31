#ifndef SENI_LANG_INTERPRETER_H
#define SENI_LANG_INTERPRETER_H

#include "seni_lang_parser.h"
#include "seni_lang_env.h"

seni_var *evaluate(seni_env *env, word_lookup *wl, seni_node *ast);

#endif
