#include "seni_lang_word_lookup.h"

#include <string.h>
#include <stdlib.h>


bool string_compare(char* a, char *b)
{
#if defined(_WIN32)
  return _stricmp(a, b) == 0;
#else
  return strcasecmp(a, b) == 0;
#endif
}

void cpy_len(char *src, char **dst, size_t len)
{
  char *c = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(c, src, len);
  c[len] = '\0';

  *dst = c;
}

void cpy(char *src, char **dst)
{
  size_t len = strlen(src);
  
  char *c = (char *)malloc(sizeof(char) * (len + 1));
  strncpy(c, src, len);
  c[len] = '\0';

  *dst = c;
}

/* returns 0 if not found */
i32 lookup_reserved_name(word_lookup *nl, char *string, size_t len)
{
  i32 i = 0;
  for (i = 0; i < nl->reserved_words_count; i++) {
    char *name = nl->reserved_words[i];
    bool found = true;
    /* can't use string_compare since 'string' could be a substring */
    size_t j = 0;
    for (j = 0; j < len; j++) {
      if (name[j] == '\0' || (name[j] != string[j])) {
        found = false;
        break;
      }
    }
    /* searched all of 'string' and the early exit wasn't triggered */
    if (name[j] == '\0' && found == true) {
      return i + RESERVED_WORD_START; // add offset
    }
  }

  return 0;
}

i32 word_lookup_or_add(word_lookup *nl, char *string, size_t len)
{
  i32 reserved = lookup_reserved_name(nl, string, len);
  if (reserved != 0) {
    return reserved;
  }
  
  i32 i = 0;
  for (i = 0; i < nl->words_count; i++) {
    char *name = nl->words[i];
    bool found = true;
    /* can't use string_compare since 'string' could be a substring */
    size_t j = 0;
    for (j = 0; j < len; j++) {
      if (name[j] == '\0' || (name[j] != string[j])) {
        found = false;
        break;
      }
    }
    /* searched all of 'string' and the early exit wasn't triggered */
    if (name[j] == '\0' && found == true) {
      return i;
    }
  }
  
  /* string is not in the table and there's no room for another entry */
  if (i >= MAX_WORD_LOOKUPS) {
    return -1;
  }

  // the string is not in the lookup table, so add it
  cpy_len(string, &(nl->words[i]), len);
  nl->words_count++;

  return i;
}


void word_lookup_add_reserved_words(word_lookup *nl)
{
  nl->reserved_words_count = 0;
  cpy("+", &(nl->reserved_words[nl->reserved_words_count++]));
  cpy("list", &(nl->reserved_words[nl->reserved_words_count++]));
  cpy("loop", &(nl->reserved_words[nl->reserved_words_count++]));
  cpy("fn", &(nl->reserved_words[nl->reserved_words_count++]));
}

void word_lookup_free_reserved_words(word_lookup *nl)
{
  for( int i = 0; i < MAX_WORD_LOOKUPS; i++) {
    if (nl->reserved_words[i]) {
      free(nl->reserved_words[i]);
    }
    nl->reserved_words[i] = 0;      
  }
  nl->reserved_words_count = 0;
}

void word_lookup_free_words(word_lookup *nl)
{
  for( int i = 0; i < MAX_WORD_LOOKUPS; i++) {
    if (nl->words[i]) {
      free(nl->words[i]);
    }
    nl->words[i] = 0;      
  }
  nl->words_count = 0;
}
