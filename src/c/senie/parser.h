#pragma once

#include "types.h"

void parser_subsystem_startup();
void parser_subsystem_shutdown();

senie_node* parser_parse(senie_word_lut* word_lut, char* s);
void        parser_return_nodes_to_pool(senie_node* nodes);
