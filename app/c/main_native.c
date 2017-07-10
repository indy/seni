#include "stdio.h"

#include "seni_lang.h"
#include "seni_vm_parser.h"
#include "seni_vm_compiler.h"
#include "seni_vm_interpreter.h"
#include "seni_shapes.h"
#include "seni_uv_mapper.h"
#include "seni_bind.h"

#include "seni_printf.h"
#include "seni_timing.h"

int main(void)
{
  TIMING_UNIT start = get_timing();
  
  if (INAME_NUMBER_OF_KNOWN_WORDS >= NATIVE_START) {
    SENI_ERROR("WARNING: keywords are overwriting into NATIVE_START area");
    return;
  }

  //char *expr = "(fn (k) (+ 9 8)) (k) (rect)";
  // char *expr2 = "(rect position: [(/ canvas/width 2) (/ canvas/height 2)] width: canvas/width height: canvas/height colour: (col/rgb r: 0 g: 0 b: 0 alpha: 1)) (wash colour: (col/rgb r: 0.15 g: 0.1 b: 0.2 alpha: 0.1)) (stroked-bezier-rect position: [(/ canvas/width 2) 600] width: 800 height: 600 colour-volatility: {90 (gen/int min: 0 max: 60)} colour: (col/rgb r: {0.8 (gen/scalar)} g: {0.0 (gen/scalar)} b: {0.0 (gen/scalar)} alpha: 0.3) volatility: {2 (gen/int min: 0 max: 100)} iterations: {90 (gen/int min: 0 max: 100)} seed: {40 (gen/int min: 0 max: 100)} overlap: {90.0 (gen/scalar min: 0 max: 5.0)}) (stroked-bezier-rect position: [(/ canvas/width 2) 200] width: 760 height: 200 colour-volatility: {9 (gen/int min: 0 max: 50)} colour: (col/rgb r: {0.15 (gen/scalar)} g: {0.10 (gen/scalar)} b: {0.20 (gen/scalar)} alpha: 0.3) volatility: {20 (gen/int min: 0 max: 100)} iterations: {90 (gen/int min: 0 max: 100)} seed: {42 (gen/int min: 0 max: 100)} overlap: {10.0 (gen/scalar min: 0 max: 5.0)}) (fn (wash variation: 200 line-width: 70 line-segments: 5 colour: (col/rgb r: 0.627 g: 0.627 b: 0.627 alpha: 0.4) seed: 272) (define w/3 (/ canvas/width 3) h/3 (/ canvas/height 3)) (loop (h from: -20 to: 1020 increment: 20) (bezier tessellation: line-segments line-width: line-width coords: [[0 (wash-wobble x: 0 y: h z: seed s: variation)] [w/3 (wash-wobble x: w/3 y: h z: seed s: variation)] [(* w/3 2) (wash-wobble x: (* w/3 2) y: h z: seed s: variation)] [canvas/width (wash-wobble x: canvas/width y: h z: seed s: variation)]] colour: colour) (bezier tessellation: line-segments line-width: line-width coords: [[(wash-wobble x: 0 y: h z: seed s: variation) 0] [(wash-wobble x: h/3 y: h z: seed s: variation) h/3] [(wash-wobble x: (* h/3 2) y: h z: seed s: variation) (* h/3 2)] [(wash-wobble x: canvas/height y: h z: seed s: variation) canvas/height]] colour: colour))) (fn (wash-wobble x: 0 y: 0 z: 0 s: 1) (+ y (* s (prng/perlin x: x y: y z: z))))";

  char *expr = "(define num-squares-to-render 15 gap-size 30 num-squares (+ 2 num-squares-to-render) num-gaps (+ num-squares 1) square-size (/ (- canvas/width (* gap-size num-gaps)) num-squares)) (wash variation: 40 line-width: 25 line-segments: 5 colour: (col/rgb r: 1.0 g: 1.0 b: 0.9)) (loop (y from: 1 to: (- num-squares 1)) (loop (x from: 1 to: (- num-squares 1)) (define x-pos (map-to-position at: x) y-pos (map-to-position at: y)) (stroked-bezier-rect position: [x-pos y-pos] colour-volatility: 20 volatility: (/ (math/distance vec1: [(/ canvas/width 2) (/ canvas/height 2)] vec2: [x-pos y-pos]) 100) seed: (+ x (* y num-squares)) width: square-size height: square-size colour: (col/rgb r: 1.0 g: 0.0 b: 0.4 alpha: 1.0)))) (fn (map-to-position at: 0) (+ (* (+ gap-size square-size) at) (/ square-size 2) gap-size)) (fn (stroked-bezier-rect position: [0 0] width: 10 height: 10 colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 alpha: 0.5) colour-volatility: 0 volatility: 0 overlap: 3 iterations: 10 seed: 343) (define [x y] position third-width (/ width 3) third-height (/ height 3) vol volatility start-x (- x (/ width 2)) start-y (- y (/ height 2)) h-delta (/ height iterations) h-strip-width (/ height iterations) half-h-strip-width (/ h-strip-width 2) v-delta (/ width iterations) v-strip-width (/ width iterations) half-v-strip-width (/ v-strip-width 2) rng (prng/build min: -1 max: 1 seed: seed) half-alpha (/ (col/get-alpha colour: colour) 2) lab-colour (col/set-alpha colour: (col/convert format: LAB colour: colour) value: half-alpha)) (loop (i to: iterations) (define [rx1 ry1 rx2 ry2 rx3 ry3 rx4 ry4] (prng/take num: 8 from: rng) lightness (+ (col/get-lab-l colour: lab-colour) (* colour-volatility (prng/take-1 from: rng))) current-colour (col/set-lab-l colour: lab-colour value: lightness)) (bezier tessellation: 10 line-width: (+ overlap h-strip-width) coords: [[(+ (+ (* rx1 vol) start-x) (* 0 third-width)) (+ (+ (* i h-delta) (* ry1 vol) start-y) half-h-strip-width)] [(+ (+ (* rx2 vol) start-x) (* 1 third-width)) (+ (+ (* i h-delta) (* ry2 vol) start-y) half-h-strip-width)] [(+ (+ (* rx3 vol) start-x) (* 2 third-width)) (+ (+ (* i h-delta) (* ry3 vol) start-y) half-h-strip-width)] [(+ (+ (* rx4 vol) start-x) (* 3 third-width)) (+ (+ (* i h-delta) (* ry4 vol) start-y) half-h-strip-width)]] colour: current-colour)) (loop (i to: iterations) (define [rx1 ry1 rx2 ry2 rx3 ry3 rx4 ry4] (prng/take num: 8 from: rng) lightness (+ (col/get-lab-l colour: lab-colour) (* colour-volatility (prng/take-1 from: rng))) current-colour (col/set-lab-l colour: lab-colour value: lightness)) (bezier tessellation: 10 line-width: (+ overlap v-strip-width) coords: [[(+ (+ (* i v-delta) (* rx1 vol) start-x) half-v-strip-width) (+ (+ (* ry1 vol) start-y) (* 0 third-height))] [(+ (+ (* i v-delta) (* rx2 vol) start-x) half-v-strip-width) (+ (+ (* ry2 vol) start-y) (* 1 third-height))] [(+ (+ (* i v-delta) (* rx3 vol) start-x) half-v-strip-width) (+ (+ (* ry3 vol) start-y) (* 2 third-height))] [(+ (+ (* i v-delta) (* rx4 vol) start-x) half-v-strip-width) (+ (+ (* ry4 vol) start-y) (* 3 third-height))]] colour: current-colour))) (fn (wash variation: 200 line-width: 70 line-segments: 5 colour: (col/rgb r: 0.627 g: 0.627 b: 0.627 alpha: 0.4) seed: 272) (define w/3 (/ canvas/width 3) h/3 (/ canvas/height 3)) (loop (h from: -20 to: 1020 increment: 20) (bezier tessellation: line-segments line-width: line-width coords: [[0 (wash-wobble x: 0 y: h z: seed s: variation)] [w/3 (wash-wobble x: w/3 y: h z: seed s: variation)] [(* w/3 2) (wash-wobble x: (* w/3 2) y: h z: seed s: variation)] [canvas/width (wash-wobble x: canvas/width y: h z: seed s: variation)]] colour: colour) (bezier tessellation: line-segments line-width: line-width coords: [[(wash-wobble x: 0 y: h z: seed s: variation) 0] [(wash-wobble x: h/3 y: h z: seed s: variation) h/3] [(wash-wobble x: (* h/3 2) y: h z: seed s: variation) (* h/3 2)] [(wash-wobble x: canvas/height y: h z: seed s: variation) canvas/height]] colour: colour))) (fn (wash-wobble x: 0 y: 0 z: 0 s: 1) (+ y (* s (prng/perlin x: x y: y z: z))))";

  // construct
  //
  seni_vm *vm = vm_construct(STACK_SIZE,HEAP_SIZE);
  seni_word_lut *wl = wlut_allocate();
  seni_env *e = env_construct();
  declare_bindings(wl, e);
  seni_shapes_init_globals();
  init_uv_mapper();

  // parse/compile
  //
  seni_node *ast = parser_parse(wl, expr);
  seni_program *prog = program_construct(1024, wl, e);
  compiler_compile(ast, prog);

  // prepare storage for vertices
  //
  int max_vertices = 10000;
  seni_render_data *render_data = render_data_construct(max_vertices);
  add_render_packet(render_data);
  vm->render_data = render_data;

  // execute
  //
  vm_interpret(vm, prog);

  // stats
  //
  i32 num_vertices = 0;
  for (int i = 0; i < render_data->num_render_packets; i++) {
    seni_render_packet *render_packet = get_render_packet(vm->render_data, i);
    num_vertices += render_packet->num_vertices;
  }
  SENI_PRINT("\ntime taken %.2f ms", timing_delta_from(start));
  SENI_PRINT("rendered %d vertices in %d render packets", num_vertices, render_data->num_render_packets);

  // free memory
  //
  wlut_free(wl);
  parser_free_nodes(ast);
  program_free(prog);
  env_free(e);
  vm_free(vm);
  free_uv_mapper();
  
  return 0;
}
