#pragma once

#include "types.h"

struct sen_result_node {
  sen_node* result;
  sen_error error;
};

struct sen_result_i32 {
  i32       result;
  sen_error error;
};

void parser_subsystem_startup();
void parser_subsystem_shutdown();

sen_result_node parser_parse(sen_word_lut* word_lut, char* s);
void            parser_return_nodes_to_pool(sen_node* nodes);
