#pragma once

#include "seni_types.h"

void parser_subsystem_startup();
void parser_subsystem_shutdown();

seni_node     *parser_parse(seni_word_lut *word_lut, char *s);
void           parser_return_nodes_to_pool(seni_node *nodes);

