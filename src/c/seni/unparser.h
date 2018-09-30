#pragma once

#include "types.h"

bool unparse(sen_cursor* cursor, sen_word_lut* word_lut, sen_node* ast,
             sen_genotype* genotype);

// unparse but remove the curly brackets and only show the default values
//
bool simplified_unparse(sen_cursor* cursor, sen_word_lut* word_lut,
                        sen_node* ast);
