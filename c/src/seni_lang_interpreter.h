#ifndef SENI_LANG_INTERPRETER_H
#define SENI_LANG_INTERPRETER_H

#include "seni_lang_parser.h"
#include "seni_lang_env.h"

void interpreter_declare_keywords(word_lut *wl);
seni_var *evaluate(seni_env *env, word_lut *wl, seni_node *ast);



#endif
