#ifndef SENI_CONTAINERS_H
#define SENI_CONTAINERS_H

#include "seni_types.h"

// https://troydhanson.github.io/uthash/userguide.html
#include "uthash/uthash.h"

// https://troydhanson.github.io/uthash/utlist.html
#include "uthash/utlist.h"

typedef struct
{
  int id;                    /* key */
  f32 ff;
  char name[10];
  UT_hash_handle hh;         /* makes this structure hashable */
} my_struct;



#endif
