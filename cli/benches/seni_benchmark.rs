use core::*;

use criterion::black_box;
use criterion::{Criterion, ParameterizedBenchmark};
use criterion::{criterion_group, criterion_main};

fn parse_script_old(source: &str) {
    let (_ast, _word_lut) = parse(&source).unwrap();
}

fn parse_script_new(source: &str) {
    let (_ast, _word_lut) = parse(&source).unwrap();
}

fn compile_ast(ast: &[Node]) {
    let _program = compile_program(&ast).unwrap();
}

fn interpret_script(source: &str) {
    let mut vm = Vm::new();
    let (ast, _word_lut) = parse(&source).unwrap();
    let program = compile_program(&ast).unwrap();

    vm.reset();

    // setup the env with the global variables in preamble
    let preamble = compile_preamble().unwrap();
    vm.interpret(&preamble).unwrap();

    vm.ip = 0;
    vm.interpret(&program).unwrap();
    let _res = vm.top_stack_value().unwrap();
}


fn benchmark_parse_comparison(c: &mut Criterion) {
    let small_src = "(define
  coords1 [{[-194.420 69.683] (gen/stray-2d from: [-194.420 69.683] by: [5 5])}
           {[396.583 297.035] (gen/stray-2d from: [396.583 297.035] by: [5 5])}
           {[349.477 358.412] (gen/stray-2d from: [349.477 358.412] by: [5 5])}
           {[-182.800 -180.599] (gen/stray-2d from: [-182.800 -180.599] by: [5 5])}]

  num-copies {24 (gen/int min: 22 max: 26)}
  squish (interp/build from: [0 (- num-copies 1)]
                       to: [{1.3 (gen/scalar min: 1.0 max: 1.5)} {1.53 (gen/scalar min: 1.4 max: 1.6)}]))

  (+ 3 4)
";
    let input = vec![small_src];

    c.bench(
        "parse",
        ParameterizedBenchmark::new("old-method", move |b, i| b.iter(|| parse_script_old(black_box(*i))), input)
            .with_function("new-method", |b, i| b.iter(|| parse_script_new(black_box(*i)))),
    );

}

fn benchmark_compile(c: &mut Criterion) {
    let source = "(rect position: [500 500] width: 100 height: 200 colour: red)";
    let (ast, _word_lut) = parse(&source).unwrap();

    c.bench_function("compile", move |b| b.iter(|| compile_ast(&ast)));
}

fn benchmark_interpret(c: &mut Criterion) {
    let source = "(rect position: [500 500] width: 100 height: 200 colour: red)";

    c.bench_function("interpret", move |b| b.iter(|| interpret_script(&source)));
}

criterion_group!(benches,
                 benchmark_parse_comparison,
                 benchmark_interpret,
                 benchmark_compile);
criterion_main!(benches);
