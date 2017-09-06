#pragma once
  
// macros that create a pool allocator for a given struct.
// the requirements for the struct are:
// - it should contain 'next' and 'prev' pointers
// - there should be constructor and destructor functions
//
// e.g. 
//
//   struct seni_item {
//     struct seni_item *next;
//     struct seni_item *prev;
//   };
//
//   typedef struct seni_item seni_item;
//
//   void item_constructor(seni_item *item) { }
//   void item_destructor(seni_item *item) { }
//
//
// if this is defined in a source file then the pool can be
// created by calling the SENI_POOL macro:
//
//   SENI_POOL(seni_item, item);
//
// the first argument is the name of the struct and the second
// is the prefix for the generated functions.
//
// structures and functions to use:
//   seni_item_pool : structure used to manage the pool
//
//   struct seni_item_pool *item_pool_allocate(i32 num_slabs,
//                                             i32 slab_size,
//                                             i32 max_slabs_allowed);
//   void item_pool_free(struct seni_item_pool *item_pool);
//   seni_item *item_pool_get(struct seni_item_pool *item_pool);
//   void item_pool_return(struct seni_item_pool *item_pool, seni_item *item);
//
// implementation detail structures and functions that shouldn't be called:
//   seni_item_slab : housekeeping structure for managing blocks of memory
//
//   struct seni_item_slab *item_slab_allocate(i32 num_items);
//   void item_slab_free(struct seni_item_slab *item_slab);
//   bool item_pool_add_slab(struct seni_item_pool *item_pool);
//
// if the pool structures have to be split into a header file
// declaration and a source file definition then use the
// SENI_POOL_DECL and SENI_POOL_IMPL macros:
//
// in the header file:
//   #include "seni_pool_macro.h"
//   SENI_POOL_DECL(seni_item, item);
//
// in the source file:
//   #include "seni_pool_macro.h"
//   #include "lib/utlist.h"
//   SENI_POOL_IMPL(seni_item, item);

#define SENI_POOL(ITEM, ITEM_NAME) \
  SENI_POOL_DECL(ITEM, ITEM_NAME); \
  SENI_POOL_IMPL(ITEM, ITEM_NAME)

#define SENI_POOL_DECL(ITEM, ITEM_NAME) struct ITEM##_slab {            \
    ITEM *ITEM_NAME##s;                                                 \
    i32 slab_size;                                                      \
    struct ITEM##_slab *next;                                           \
    struct ITEM##_slab *prev;                                           \
  };                                                                    \
  struct ITEM##_pool {                                                  \
    struct ITEM##_slab *ITEM_NAME##_slabs;                              \
    i32 slab_size;                                                      \
    i32 num_slabs;                                                      \
    i32 max_slabs_allowed;                                              \
                                                                        \
    ITEM *available;                                                    \
                                                                        \
    i32 get_count;                                                      \
    i32 return_count;                                                   \
    i32 high_water_mark;                                                \
    i32 current_water_mark;                                             \
  };                                                                    \
                                                                        \
  extern struct ITEM##_slab *ITEM_NAME##_slab_allocate(i32 num_items);  \
  extern void ITEM_NAME##_slab_free(struct ITEM##_slab *ITEM_NAME##_slab); \
  extern bool ITEM_NAME##_pool_add_slab(struct ITEM##_pool *ITEM_NAME##_pool); \
  extern void ITEM_NAME##_pool_free(struct ITEM##_pool *ITEM_NAME##_pool); \
  extern struct ITEM##_pool *ITEM_NAME##_pool_allocate(i32 num_slabs, i32 slab_size, i32 max_slabs_allowed); \
  extern ITEM *ITEM_NAME##_pool_get(struct ITEM##_pool *ITEM_NAME##_pool); \
  extern void ITEM_NAME##_pool_return(struct ITEM##_pool *ITEM_NAME##_pool, ITEM *ITEM_NAME);

#define SENI_POOL_IMPL(ITEM, ITEM_NAME) struct ITEM##_slab *ITEM_NAME##_slab_allocate(i32 num_items) \
  {                                                                     \
    struct ITEM##_slab *ITEM_NAME##_slab = (struct ITEM##_slab *)calloc(1, sizeof(struct ITEM##_slab)); \
                                                                        \
    ITEM_NAME##_slab->slab_size = num_items;                            \
    ITEM_NAME##_slab->ITEM_NAME##s = (ITEM *)calloc(num_items, sizeof(ITEM)); \
                                                                        \
    ITEM *ITEM_NAME = ITEM_NAME##_slab->ITEM_NAME##s;                   \
    for (i32 i = 0; i < ITEM_NAME##_slab->slab_size; i++) {             \
      ITEM_NAME##_constructor(ITEM_NAME);                               \
      ITEM_NAME++;                                                      \
    }                                                                   \
                                                                        \
    return ITEM_NAME##_slab;                                            \
  }                                                                     \
                                                                        \
  void ITEM_NAME##_slab_free(struct ITEM##_slab *ITEM_NAME##_slab)      \
  {                                                                     \
    ITEM *ITEM_NAME = ITEM_NAME##_slab->ITEM_NAME##s;                   \
    for (i32 i = 0; i < ITEM_NAME##_slab->slab_size; i++) {             \
      ITEM_NAME##_destructor(ITEM_NAME);                                \
      ITEM_NAME++;                                                      \
    }                                                                   \
                                                                        \
    free(ITEM_NAME##_slab);                                             \
  }                                                                     \
                                                                        \
  bool ITEM_NAME##_pool_add_slab(struct ITEM##_pool *ITEM_NAME##_pool)  \
  {                                                                     \
    if (ITEM_NAME##_pool->num_slabs >= ITEM_NAME##_pool->max_slabs_allowed) { \
      SENI_ERROR("will not allocate more than %d ITEM_NAME##_slabs", ITEM_NAME##_pool->max_slabs_allowed); \
      return false;                                                     \
    }                                                                   \
                                                                        \
    struct ITEM##_slab *ITEM_NAME##_slab = ITEM_NAME##_slab_allocate(ITEM_NAME##_pool->slab_size); \
    DL_APPEND(ITEM_NAME##_pool->ITEM_NAME##_slabs, ITEM_NAME##_slab);   \
                                                                        \
    ITEM *ITEM_NAME = ITEM_NAME##_slab->ITEM_NAME##s;                   \
    for (i32 i = 0; i < ITEM_NAME##_pool->slab_size; i++) {             \
      DL_APPEND(ITEM_NAME##_pool->available, ITEM_NAME);                \
      ITEM_NAME++;                                                      \
    }                                                                   \
                                                                        \
    ITEM_NAME##_pool->num_slabs++;                                      \
                                                                        \
    return true;                                                        \
  }                                                                     \
                                                                        \
  void ITEM_NAME##_pool_free(struct ITEM##_pool *ITEM_NAME##_pool)      \
  {                                                                     \
    struct ITEM##_slab *ITEM_NAME##_slab = ITEM_NAME##_pool->ITEM_NAME##_slabs; \
    struct ITEM##_slab *next;                                           \
                                                                        \
    for (i32 i = 0; i < ITEM_NAME##_pool->num_slabs; i++) {             \
      if (ITEM_NAME##_slab) {                                           \
        next = ITEM_NAME##_slab->next;                                  \
      }                                                                 \
      ITEM_NAME##_slab_free(ITEM_NAME##_slab);                          \
      ITEM_NAME##_slab = next;                                          \
    }                                                                   \
                                                                        \
    free(ITEM_NAME##_pool);                                             \
  }                                                                     \
                                                                        \
  struct ITEM##_pool *ITEM_NAME##_pool_allocate(i32 num_slabs, i32 slab_size, i32 max_slabs_allowed) \
  {                                                                     \
    struct ITEM##_pool *ITEM_NAME##_pool = (struct ITEM##_pool *)calloc(1, sizeof(struct ITEM##_pool)); \
    ITEM_NAME##_pool->slab_size = slab_size;                            \
    ITEM_NAME##_pool->max_slabs_allowed = max_slabs_allowed;            \
                                                                        \
    for(i32 i = 0; i < num_slabs; i++) {                                \
      if (!ITEM_NAME##_pool_add_slab(ITEM_NAME##_pool)) {               \
        ITEM_NAME##_pool_free(ITEM_NAME##_pool);                        \
        return NULL;                                                    \
      }                                                                 \
    }                                                                   \
                                                                        \
    return ITEM_NAME##_pool;                                            \
  }                                                                     \
                                                                        \
  ITEM *ITEM_NAME##_pool_get(struct ITEM##_pool *ITEM_NAME##_pool)      \
  {                                                                     \
    if (ITEM_NAME##_pool->available == NULL) {                          \
      if (!ITEM_NAME##_pool_add_slab(ITEM_NAME##_pool)) {               \
        SENI_ERROR("cannot add more than %d ITEM##_slabs", ITEM_NAME##_pool->max_slabs_allowed); \
        return NULL;                                                    \
      }                                                                 \
    }                                                                   \
                                                                        \
    ITEM *head = ITEM_NAME##_pool->available;                           \
    DL_DELETE(ITEM_NAME##_pool->available, head);                       \
                                                                        \
    head->next = NULL;                                                  \
    head->prev = NULL;                                                  \
                                                                        \
    ITEM_NAME##_pool->get_count++;                                      \
    ITEM_NAME##_pool->current_water_mark++;                             \
    if (ITEM_NAME##_pool->current_water_mark > ITEM_NAME##_pool->high_water_mark) { \
      ITEM_NAME##_pool->high_water_mark = ITEM_NAME##_pool->current_water_mark; \
    }                                                                   \
                                                                        \
    return head;                                                        \
  }                                                                     \
                                                                        \
  void ITEM_NAME##_pool_return(struct ITEM##_pool *ITEM_NAME##_pool, ITEM *ITEM_NAME) \
  {                                                                     \
    ITEM_NAME->next = NULL;                                             \
    ITEM_NAME->prev = NULL;                                             \
                                                                        \
    DL_APPEND(ITEM_NAME##_pool->available, ITEM_NAME);                  \
                                                                        \
    ITEM_NAME##_pool->return_count++;                                   \
    ITEM_NAME##_pool->current_water_mark--;                             \
  }




