#pragma once

// macro that create a pool allocator for a given struct.
// the requirements for the struct are:
// - it should contain 'next' and 'prev' pointers
// - there should be cleanup function
//
// e.g.
//
//   typedef struct senie_item {
//     struct senie_item *next;
//     struct senie_item *prev;
//   } senie_item;
//
//   void item_cleanup(senie_item *item) { }
//
// if this is defined in a source file then the pool can be
// created by calling the SENIE_POOL macro:
//
// SENIE_POOL(senie_item, item)
//
// the first argument is the name of the struct and the second
// is the prefix for the generated functions.
//
// structures and functions to use:
//   senie_item_pool : structure used to manage the pool
//
//   struct senie_item_pool *item_pool_allocate(i32 num_slabs,
//                                             i32 slab_size,
//                                             i32 max_slabs_allowed);
//   void item_pool_free(struct senie_item_pool *item_pool);
//   senie_item *item_pool_get(struct senie_item_pool *item_pool);
//   void item_pool_return(struct senie_item_pool *item_pool, senie_item *item);
//
// *** RECOMMENDATIONS ***
//
// have global level pool structures:
//
//   struct senie_item_pool *g_item_pool;
//
// have system level startup/shutdown functions for pool allocation/freeing:
//
//   void some_system_startup()
//   {
//     g_item_pool = item_pool_allocate(1, 20, 10);
//   }
//
//   void some_system_shutdown()
//   {
//     item_pool_free(g_item_pool);
//   }
//
// implement functions to get and return items from/to the pool:
//
//   senie_item *item_get_from_pool()
//   {
//     senie_item *item = item_pool_get(g_item_pool);
//     return item;
//   }
//
//   void item_return_to_pool(senie_item *item)
//   {
//     item_pool_return(g_item_pool, item);
//   }
//
// the rest of the code will use these functions and not refer
// to any of the generated functions or global pool variables

#define SENIE_POOL(ITEM, ITEM_NAME)                                                                \
  struct ITEM##_slab {                                                                             \
    ITEM*               ITEM_NAME##s;                                                              \
    i32                 slab_size;                                                                 \
    struct ITEM##_slab* next;                                                                      \
    struct ITEM##_slab* prev;                                                                      \
  };                                                                                               \
  struct ITEM##_pool {                                                                             \
    struct ITEM##_slab* ITEM_NAME##_slabs;                                                         \
    i32                 slab_size;                                                                 \
    i32                 num_slabs;                                                                 \
    i32                 max_slabs_allowed;                                                         \
                                                                                                   \
    ITEM* available;                                                                               \
                                                                                                   \
    i32 get_count;                                                                                 \
    i32 return_count;                                                                              \
    i32 high_water_mark;                                                                           \
    i32 current_water_mark;                                                                        \
  };                                                                                               \
                                                                                                   \
  extern struct ITEM##_slab* ITEM_NAME##_slab_allocate(i32 num_items);                             \
  extern void                ITEM_NAME##_slab_free(struct ITEM##_slab* ITEM_NAME##_slab);          \
  extern bool                ITEM_NAME##_pool_add_slab(struct ITEM##_pool* ITEM_NAME##_pool);      \
  extern void                ITEM_NAME##_pool_free(struct ITEM##_pool* ITEM_NAME##_pool);          \
  extern struct ITEM##_pool* ITEM_NAME##_pool_allocate(                                            \
      i32 num_slabs, i32 slab_size, i32 max_slabs_allowed);                                        \
  extern ITEM* ITEM_NAME##_pool_get(struct ITEM##_pool* ITEM_NAME##_pool);                         \
  extern void  ITEM_NAME##_pool_return(struct ITEM##_pool* ITEM_NAME##_pool, ITEM* ITEM_NAME);     \
                                                                                                   \
  struct ITEM##_slab* ITEM_NAME##_slab_allocate(i32 num_items) {                                   \
    struct ITEM##_slab* ITEM_NAME##_slab =                                                         \
        (struct ITEM##_slab*)calloc(1, sizeof(struct ITEM##_slab));                                \
                                                                                                   \
    ITEM_NAME##_slab->slab_size    = num_items;                                                    \
    ITEM_NAME##_slab->ITEM_NAME##s = (ITEM*)calloc(num_items, sizeof(ITEM));                       \
                                                                                                   \
    ITEM* ITEM_NAME = ITEM_NAME##_slab->ITEM_NAME##s;                                              \
    for (i32 i = 0; i < ITEM_NAME##_slab->slab_size; i++) {                                        \
      ITEM_NAME++;                                                                                 \
    }                                                                                              \
                                                                                                   \
    return ITEM_NAME##_slab;                                                                       \
  }                                                                                                \
                                                                                                   \
  void ITEM_NAME##_slab_free(struct ITEM##_slab* ITEM_NAME##_slab) {                               \
    ITEM* ITEM_NAME = ITEM_NAME##_slab->ITEM_NAME##s;                                              \
    for (i32 i = 0; i < ITEM_NAME##_slab->slab_size; i++) {                                        \
      ITEM_NAME##_cleanup(ITEM_NAME);                                                              \
      ITEM_NAME++;                                                                                 \
    }                                                                                              \
                                                                                                   \
    free(ITEM_NAME##_slab);                                                                        \
  }                                                                                                \
                                                                                                   \
  bool ITEM_NAME##_pool_add_slab(struct ITEM##_pool* ITEM_NAME##_pool) {                           \
    if (ITEM_NAME##_pool->num_slabs >= ITEM_NAME##_pool->max_slabs_allowed) {                      \
      const char* slab_name = #ITEM_NAME "_slabs";                                                 \
      SENIE_ERROR(                                                                                 \
          "will not allocate more than %d %s", ITEM_NAME##_pool->max_slabs_allowed, slab_name);    \
      return false;                                                                                \
    }                                                                                              \
                                                                                                   \
    struct ITEM##_slab* ITEM_NAME##_slab = ITEM_NAME##_slab_allocate(ITEM_NAME##_pool->slab_size); \
    DL_APPEND(ITEM_NAME##_pool->ITEM_NAME##_slabs, ITEM_NAME##_slab);                              \
                                                                                                   \
    ITEM* ITEM_NAME = ITEM_NAME##_slab->ITEM_NAME##s;                                              \
    for (i32 i = 0; i < ITEM_NAME##_pool->slab_size; i++) {                                        \
      DL_APPEND(ITEM_NAME##_pool->available, ITEM_NAME);                                           \
      ITEM_NAME++;                                                                                 \
    }                                                                                              \
                                                                                                   \
    ITEM_NAME##_pool->num_slabs++;                                                                 \
                                                                                                   \
    return true;                                                                                   \
  }                                                                                                \
                                                                                                   \
  void ITEM_NAME##_pool_free(struct ITEM##_pool* ITEM_NAME##_pool) {                               \
    struct ITEM##_slab* ITEM_NAME##_slab = ITEM_NAME##_pool->ITEM_NAME##_slabs;                    \
    struct ITEM##_slab* next             = NULL;                                                   \
                                                                                                   \
    for (i32 i = 0; i < ITEM_NAME##_pool->num_slabs; i++) {                                        \
      if (ITEM_NAME##_slab) {                                                                      \
        next = ITEM_NAME##_slab->next;                                                             \
      } else {                                                                                     \
        SENIE_ERROR("slab unexpectedly null");                                                     \
        break;                                                                                     \
      }                                                                                            \
      ITEM_NAME##_slab_free(ITEM_NAME##_slab);                                                     \
      ITEM_NAME##_slab = next;                                                                     \
    }                                                                                              \
                                                                                                   \
    free(ITEM_NAME##_pool);                                                                        \
  }                                                                                                \
                                                                                                   \
  struct ITEM##_pool* ITEM_NAME##_pool_allocate(                                                   \
      i32 num_slabs, i32 slab_size, i32 max_slabs_allowed) {                                       \
    struct ITEM##_pool* ITEM_NAME##_pool =                                                         \
        (struct ITEM##_pool*)calloc(1, sizeof(struct ITEM##_pool));                                \
    ITEM_NAME##_pool->slab_size         = slab_size;                                               \
    ITEM_NAME##_pool->max_slabs_allowed = max_slabs_allowed;                                       \
                                                                                                   \
    for (i32 i = 0; i < num_slabs; i++) {                                                          \
      if (!ITEM_NAME##_pool_add_slab(ITEM_NAME##_pool)) {                                          \
        ITEM_NAME##_pool_free(ITEM_NAME##_pool);                                                   \
        return NULL;                                                                               \
      }                                                                                            \
    }                                                                                              \
                                                                                                   \
    return ITEM_NAME##_pool;                                                                       \
  }                                                                                                \
                                                                                                   \
  ITEM* ITEM_NAME##_pool_get(struct ITEM##_pool* ITEM_NAME##_pool) {                               \
    if (ITEM_NAME##_pool->available == NULL) {                                                     \
      if (!ITEM_NAME##_pool_add_slab(ITEM_NAME##_pool)) {                                          \
        const char* slab_name = #ITEM_NAME "_slabs";                                               \
        SENIE_ERROR("cannot add more than %d %s", ITEM_NAME##_pool->max_slabs_allowed, slab_name); \
        return NULL;                                                                               \
      }                                                                                            \
    }                                                                                              \
                                                                                                   \
    ITEM* head = ITEM_NAME##_pool->available;                                                      \
    DL_DELETE(ITEM_NAME##_pool->available, head);                                                  \
                                                                                                   \
    head->next = NULL;                                                                             \
    head->prev = NULL;                                                                             \
                                                                                                   \
    ITEM_NAME##_pool->get_count++;                                                                 \
    ITEM_NAME##_pool->current_water_mark++;                                                        \
    if (ITEM_NAME##_pool->current_water_mark > ITEM_NAME##_pool->high_water_mark) {                \
      ITEM_NAME##_pool->high_water_mark = ITEM_NAME##_pool->current_water_mark;                    \
    }                                                                                              \
                                                                                                   \
    return head;                                                                                   \
  }                                                                                                \
                                                                                                   \
  void ITEM_NAME##_pool_return(struct ITEM##_pool* ITEM_NAME##_pool, ITEM* ITEM_NAME) {            \
    ITEM_NAME->next = NULL;                                                                        \
    ITEM_NAME->prev = NULL;                                                                        \
                                                                                                   \
    DL_APPEND(ITEM_NAME##_pool->available, ITEM_NAME);                                             \
                                                                                                   \
    ITEM_NAME##_pool->return_count++;                                                              \
    ITEM_NAME##_pool->current_water_mark--;                                                        \
  }                                                                                                \
                                                                                                   \
  void ITEM_NAME##_pool_pretty_print(struct ITEM##_pool* ITEM_NAME##_pool) {                       \
    const char* item_name = #ITEM_NAME;                                                            \
    SENIE_LOG("%s_pool:"                                                                           \
              "\tslab_size: %d num_slabs: %d max_slabs_allowed %d"                                 \
              "\tget_count: %d return_count: %d"                                                   \
              "\thigh_water_mark: %d current_water_mark: %d",                                      \
              item_name,                                                                           \
              ITEM_NAME##_pool->slab_size,                                                         \
              ITEM_NAME##_pool->num_slabs,                                                         \
              ITEM_NAME##_pool->max_slabs_allowed,                                                 \
              ITEM_NAME##_pool->get_count,                                                         \
              ITEM_NAME##_pool->return_count,                                                      \
              ITEM_NAME##_pool->high_water_mark,                                                   \
              ITEM_NAME##_pool->current_water_mark);                                               \
  }
