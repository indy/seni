#ifndef SENI_LANG_WORD_LOOKUP_H
#define SENI_LANG_WORD_LOOKUP_H

#include "seni_types.h"

// 2 << 6 == 64
#define MAX_WORD_LOOKUPS (2 << 6)
#define MAX_KEYWORD_LOOKUPS MAX_WORD_LOOKUPS
#define KEYWORD_START MAX_WORD_LOOKUPS


/* word lookup table */
typedef struct word_lut {
  // filled in by interpreter: add_keywords_to_word_lookup
  char *keywords[MAX_KEYWORD_LOOKUPS];  
  i32 keywords_count;
  
  char *words[MAX_WORD_LOOKUPS];
  i32 words_count;
} word_lut;


word_lut *word_lookup_allocate();
void word_lookup_free(word_lut *wlut);

i32 word_lookup_or_add(word_lut *wlut, char *string, size_t len);

char *word_lookup_i32(word_lut *wlut, i32 index);

#endif
