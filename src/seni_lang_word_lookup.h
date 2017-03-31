#ifndef SENI_LANG_WORD_LOOKUP_H
#define SENI_LANG_WORD_LOOKUP_H

#include <stdio.h>
#include "seni_types.h"

#define MAX_WORD_LOOKUPS 64
#define RESERVED_WORD_START MAX_WORD_LOOKUPS

typedef struct word_lookup {
  char *reserved_words[MAX_WORD_LOOKUPS];  
  i32 reserved_words_count;
  
  char *words[MAX_WORD_LOOKUPS];
  i32 words_count;
} word_lookup;


i32 word_lookup_or_add(word_lookup *nl, char *string, size_t len);
void word_lookup_free_words(word_lookup *nl);

void word_lookup_add_reserved_words(word_lookup *nl);
void word_lookup_free_reserved_words(word_lookup *nl);

#endif
