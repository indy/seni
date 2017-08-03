#pragma once

#include "seni_lang.h"
#include "seni_ga.h"

bool unparse(char *out, i32 out_size, seni_env *env, seni_node *ast, seni_genotype *genotype);

