#pragma once

#include "types.h"

void parser_subsystem_startup();
void parser_subsystem_shutdown();

sen_result_node parser_parse(sen_word_lut* word_lut, char* s);
void            parser_return_nodes_to_pool(sen_node* nodes);
