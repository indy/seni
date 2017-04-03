#ifndef SENI_LANG_WORD_LOOKUP_H
#define SENI_LANG_WORD_LOOKUP_H

#include <stdio.h>
#include "seni_types.h"

// 2 << 6 == 64
#define MAX_WORD_LOOKUPS (2 << 6)
#define KEYWORD_START MAX_WORD_LOOKUPS

// IMPORTANT: make sure these defines match up with word_lookup_add_keywords
//
#define KEYWORD_PLUS     (KEYWORD_START + 0)
#define KEYWORD_MINUS    (KEYWORD_START + 1)
#define KEYWORD_MULTIPLY (KEYWORD_START + 2)
#define KEYWORD_DIVIDE   (KEYWORD_START + 3)
#define KEYWORD_DEFINE   (KEYWORD_START + 4)
#define KEYWORD_FN       (KEYWORD_START + 5)


/* word lookup table */
typedef struct word_lut {
  char *keywords[MAX_WORD_LOOKUPS];  
  i32 keywords_count;
  
  char *words[MAX_WORD_LOOKUPS];
  i32 words_count;
} word_lut;


i32 word_lookup_or_add(word_lut *wlut, char *string, size_t len);
void word_lookup_free_words(word_lut *wlut);

void word_lookup_add_keywords(word_lut *wlut);
void word_lookup_free_keywords(word_lut *wlut);

char *word_lookup_i32(word_lut *wlut, i32 index);

#endif
