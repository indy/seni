#include "seni_bind.h"
#include "seni_ga.h"
#include "seni_keyword_iname.h"
#include "seni_lang.h"
#include "seni_parser.h"
#include "seni_printf.h"
#include "seni_render_packet.h"
#include "seni_shapes.h"
#include "seni_timing.h"
#include "seni_uv_mapper.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"

#include "stdio.h"
#include "stdlib.h"
#include "string.h"

char *load_file(char *filename)
{
  char *ret = NULL;
  
  int file_size = 0;
  size_t read_size = 0;
  
  FILE *fp = fopen(filename, "r");  
  if (fp) {
    while (getc(fp) != EOF) {
      file_size++;
    }
    fseek(fp, 0, SEEK_SET);

    ret = (char *)calloc(file_size + 1, sizeof(char));

    read_size = fread(ret, sizeof(char), file_size, fp);

    ret[file_size] = '\0';

    if (file_size != read_size)
      {
        SENI_ERROR("file_size %d read_size %d\n", file_size, read_size);
        free(ret);
        ret = NULL;
      }
       
    fclose(fp);
  } else {
    SENI_ERROR("fopen failed");
  }

  return ret;
}

f32 percentage(f32 total, f32 element)
{
  return (100.0f / total) * element;
}

void print_timings(f32 construct, f32 compile, f32 interpret)
{
  f32 total = construct + compile + interpret;

  SENI_PRINT("total time taken : %.2f ms", total);
  if (total > 0.0f) {
    SENI_PRINT("construct time   : %.2f ms\t(%.2f%%)", construct, percentage(total, construct));
    SENI_PRINT("compile time     : %.2f ms\t(%.2f%%)", compile, percentage(total, compile));
    SENI_PRINT("interpret time   : %.2f ms\t(%.2f%%)", interpret, percentage(total, interpret));
  }
}

char *pluralise(i32 count, char *singular, char *plural)
{
  return count == 1 ? singular : plural;
}

void execute_source(char *source)
{
  // construct
  //
  TIMING_UNIT construct_start = get_timing();
  seni_vm *vm = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  seni_env *env = env_allocate();
  lang_pools_startup();
  parser_pools_startup();
  ga_pools_startup();
  seni_shapes_init_globals();
  uv_mapper_init();
  TIMING_UNIT construct_stop = get_timing();
  
  // parse/compile
  //
  TIMING_UNIT compilation_start = get_timing();

  seni_node *ast = parser_parse(env->wl, source);

  seni_program *program = compile_program(ast, MAX_PROGRAM_SIZE, env->wl);

  parser_return_nodes_to_pool(ast);

  
  TIMING_UNIT compilation_stop = get_timing();

  // execute
  //
  TIMING_UNIT interpret_start = get_timing();
  vm_debug_info_reset(vm);
  vm_interpret(vm, env, program);
  TIMING_UNIT interpret_stop = get_timing();

  // stats
  //
  i32 num_vertices = 0;
  for (int i = 0; i < vm->render_data->num_render_packets; i++) {
    seni_render_packet *render_packet = get_render_packet(vm->render_data, i);
    num_vertices += render_packet->num_vertices;
  }

  if (num_vertices != 0) {
    SENI_PRINT("\nrendered %d vertices in %d render packets",
               num_vertices, vm->render_data->num_render_packets);
    print_timings(timing_delta(construct_start, construct_stop),
                  timing_delta(compilation_start, compilation_stop),
                  timing_delta(interpret_start, interpret_stop));
  }

  // free memory
  //
  program_free(program);
  env_free(env);
  vm_free(vm);
  uv_mapper_free();
  ga_pools_shutdown();
  parser_pools_shutdown();
  lang_pools_shutdown();
}

void execute_source_with_seed(char *source, i32 seed_value)
{
  // construct
  //
  TIMING_UNIT construct_start = get_timing();
  seni_vm *vm = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  seni_env *env = env_allocate();
  lang_pools_startup();
  parser_pools_startup();
  ga_pools_startup();
  seni_shapes_init_globals();
  uv_mapper_init();
  TIMING_UNIT construct_stop = get_timing();
  
  // parse/compile
  //
  TIMING_UNIT compilation_start = get_timing();

  seni_node *ast = parser_parse(env->wl, source);

  seni_trait_list *trait_list = trait_list_compile(ast, MAX_TRAIT_PROGRAM_SIZE, env->wl);

  // using the vm to build the genes
  seni_genotype *genotype = genotype_build(vm, env, trait_list, seed_value);

  seni_program *program = compile_program_with_genotype(ast, MAX_PROGRAM_SIZE, env->wl, genotype);
  
  parser_return_nodes_to_pool(ast);

  
  TIMING_UNIT compilation_stop = get_timing();

  // execute
  //
  TIMING_UNIT interpret_start = get_timing();
  vm_debug_info_reset(vm);
  vm_interpret(vm, env, program);
  TIMING_UNIT interpret_stop = get_timing();

  // stats
  //
  i32 num_vertices = 0;
  for (int i = 0; i < vm->render_data->num_render_packets; i++) {
    seni_render_packet *render_packet = get_render_packet(vm->render_data, i);
    num_vertices += render_packet->num_vertices;
  }

  if (num_vertices != 0) {
    SENI_PRINT("\nrendered %d vertices in %d render packets",
               num_vertices, vm->render_data->num_render_packets);
    print_timings(timing_delta(construct_start, construct_stop),
                  timing_delta(compilation_start, compilation_stop),
                  timing_delta(interpret_start, interpret_stop));
  }

  i32 num_traits = trait_list_count(trait_list);
  if (num_traits > 0) {
    SENI_PRINT("%d %s", num_traits, pluralise(num_traits, "trait", "traits"));
  }


  // seni_gene *gene = genotype->genes;
  // while (gene) {
  //   var_pretty_print("gene: ", &(gene->var));
  //   gene = gene->next;
  // }
  

  // free memory
  //
  genotype_return_to_pool(genotype);
  trait_list_return_to_pool(trait_list);
  program_free(program);
  env_free(env);
  vm_free(vm);
  uv_mapper_free();
  ga_pools_shutdown();
  parser_pools_shutdown();
  lang_pools_shutdown();
}

void print_compiled_program(char *source)
{
  // construct
  seni_vm *vm = vm_allocate(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE, VERTEX_PACKET_NUM_VERTICES);
  seni_env *e = env_allocate();

  // compile program
  seni_program *program = program_compile(e, MAX_PROGRAM_SIZE, source);

  // print
  printf("%s\n", source);
  program_pretty_print(program);

  // cleanup
  program_free(program);
  env_free(e);
  vm_free(vm);
}

void print_usage()
{
#ifdef SENI_BUILD_WINDOWS
  SENI_PRINT("native.exe                            << prints usage");
  SENI_PRINT("native.exe seni\\c\\script.seni         << execute the script using defaults and give stats");
  SENI_PRINT("native.exe seni\\c\\script.seni -s 43   << execute the script using the given seed and give stats");
  SENI_PRINT("native.exe seni\\c\\script.seni -d      << debug - output the bytecode");    
#else
  SENI_PRINT("native                            << prints usage");
  SENI_PRINT("native seni/c/script.seni         << execute the script using defaults and give stats");
  SENI_PRINT("native seni/c/script.seni -s 43   << execute the script using the given seed and give stats");
  SENI_PRINT("native seni/c/script.seni -d      << debug - output the bytecode");    
#endif
}

int main(int argc, char **argv)
{
  char *source = NULL;

  if (argc == 1) {
    // invoked native without any command line options
    print_usage();
    return 0;
  }

  source = load_file(argv[1]);
  if (source == NULL) {
    return 1;
  }

  if (INAME_NUMBER_OF_KNOWN_WORDS >= NATIVE_START) {
    SENI_ERROR("WARNING: keywords are overwriting into NATIVE_START area");
    return 1;
  }

  if (argc == 2) {
    // just a filename
    execute_source(source);
  } else if (argc == 3 && strcmp(argv[2], "-d") == 0) {
    print_compiled_program(source);
  } else if (argc == 4 && strcmp(argv[2], "-s") == 0) {
    // read in the seed value
    int seed_value = atoi(argv[3]);
    execute_source_with_seed(source, (i32)seed_value);
  }

  free(source);
  
  return 0;
}



