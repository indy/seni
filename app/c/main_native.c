#include "stdio.h"
#include "stdlib.h"

#include "seni_lang.h"
#include "seni_vm_parser.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"
#include "seni_shapes.h"
#include "seni_uv_mapper.h"
#include "seni_bind.h"

#include "seni_printf.h"
#include "seni_timing.h"


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

    ret = (char *)malloc((file_size + 1) * sizeof(char));

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

void execute_source(char *source)
{
  // construct
  //
  TIMING_UNIT construct_start = get_timing();
  seni_vm *vm = vm_construct(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE);  
  seni_word_lut *wl = wlut_allocate();
  seni_env *e = env_construct();
  declare_bindings(wl, e);
  seni_shapes_init_globals();
  init_uv_mapper();

  // prepare storage for vertices
  //
  int max_vertices = 10000;
  seni_render_data *render_data = render_data_construct(max_vertices);
  add_render_packet(render_data);
  vm->render_data = render_data;
  TIMING_UNIT construct_stop = get_timing();

  // parse/compile
  //
  TIMING_UNIT compilation_start = get_timing();
  seni_node *ast = parser_parse(wl, source);
  seni_program *prog = program_construct(MAX_PROGRAM_SIZE, wl, e);
  compiler_compile(ast, prog);
  TIMING_UNIT compilation_stop = get_timing();

  // execute
  //
  TIMING_UNIT interpret_start = get_timing();
  vm_debug_info_reset(vm);
  vm_interpret(vm, prog);
  TIMING_UNIT interpret_stop = get_timing();

  // stats
  //
  i32 num_vertices = 0;
  for (int i = 0; i < render_data->num_render_packets; i++) {
    seni_render_packet *render_packet = get_render_packet(vm->render_data, i);
    num_vertices += render_packet->num_vertices;
  }

  if (num_vertices != 0) {
    SENI_PRINT("\nrendered %d vertices in %d render packets", num_vertices, render_data->num_render_packets);
    print_timings(timing_delta(construct_start, construct_stop),
                  timing_delta(compilation_start, compilation_stop),
                  timing_delta(interpret_start, interpret_stop));
  }

  // free memory
  //
  wlut_free(wl);
  parser_free_nodes(ast);
  program_free(prog);
  env_free(e);
  vm_free(vm);
  free_uv_mapper();
}



void print_compiled_program(char *source)
{
  // construct
  seni_vm *vm = vm_construct(STACK_SIZE, HEAP_SIZE, HEAP_MIN_SIZE);  
  seni_word_lut *wl = wlut_allocate();
  seni_env *e = env_construct();

  // setup
  declare_bindings(wl, e);

  // compile program
  seni_node *ast = parser_parse(wl, source);
  seni_program *prog = program_construct(MAX_PROGRAM_SIZE, wl, e);
  compiler_compile(ast, prog);

  // print
  printf("%s\n", source);
  pretty_print_program(prog);

  // cleanup
  wlut_free(wl);
  parser_free_nodes(ast);
  program_free(prog);
  env_free(e);
  vm_free(vm);
}

// print the compiled program:
// native.exe seni/c/script.seni -print

// execute script:
// native.exe seni/c/script.seni

// execute script, printing out the executed opcodes
// native.exe seni/c/script.seni -debug

int main(int argc, char **argv)
{
  char *source = NULL;
  
  if (argc > 1) {
    source = load_file(argv[1]);
  }

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
  } else if (argc == 3) {
    print_compiled_program(source);
  }

  free(source);
  
  return 0;
}
