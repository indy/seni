#pragma once

#include "types.h"

// create a sen_result_node structure which holds either
// a valid sen_node* result or a sen_error
//
RESULT_STRUCT(sen_node*, node)

void parser_subsystem_startup();
void parser_subsystem_shutdown();

sen_result_node parser_parse(sen_word_lut* word_lut, char* s);
void            parser_return_nodes_to_pool(sen_node* nodes);
