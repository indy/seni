#pragma once

#include "seni_types.h"

void parser_pools_startup();
void parser_pools_shutdown();

seni_node     *parser_parse(seni_word_lut *wlut, char *s);
void           parser_return_nodes_to_pool(seni_node *nodes);

