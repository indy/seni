#pragma once

#include "seni_lang.h"

seni_node     *parser_parse(seni_word_lut *wlut, char *s);
void           parser_free_nodes(seni_node *nodes);

