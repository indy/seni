import Parser from '../../src/lang/Parser';
import Lexer from '../../src/lang/Lexer';
import Compiler from '../../src/lang/Compiler';
import Genetic from '../../src/lang/Genetic';

describe('compiler', () => {

  function compileWithSeed(form, seed) {
    // assumes that the form will compile into a single list
    const ts = Lexer.tokenise(form).tokens;
    const ast = Parser.parse(ts).nodes;

    const traits = Genetic.buildTraits(ast);
    const genotype = Genetic.createGenotypeFromTraits(traits, seed);

    return Compiler.compile(ast, genotype);
  }

  function compile(form) {
    // assumes that the form will compile into a single list
    const ts = Lexer.tokenise(form).tokens;
    const ast = Parser.parse(ts).nodes;

    const traits = Genetic.buildTraits(ast);
    const genotype = Genetic.createGenotypeFromInitialValues(traits);

    return Compiler.compile(ast, genotype);
  }

  it('should test required functions: genotype initial', () => {
    expect(compile('(* 2 [4])').forms[0]).
      to.deep.equal(['*', 2, 4]);
  });

  it('should build a hash for the arguments', () => {
    expect(compile('(something monkey: 42)').forms[0]).
      to.deep.equal(['something', {monkey: 42}]);
  });

  it('should test required functions: genotype', () => {
    expect(compileWithSeed('(+ 2 [44 (int min: 10 max: 56)])', 100).forms[0]).
      to.deep.equal(['+', 2, 11]);

    expect(compileWithSeed('(+ 2 [44 (int min: 10 max: 56)])', 33).forms[0]).
      to.deep.equal(['+', 2, 49]);
    expect(compileWithSeed('(+ 2 [44 (int min: 10 max: 56)])', 33).forms[0]).
      to.deep.equal(['+', 2, 49]);

  });

  it('should test plus', () => {
    expect(compileWithSeed('([- (testPlus)] 2 7)', 100).forms[0]).
      to.deep.equal(['+', 2, 7]);
  });

  it('should test required functions', () => {

    expect(compile('4').forms[0]).
      to.equal(4);

    expect(compile('(* 2 4)').forms[0]).
      to.deep.equal(['*', 2, 4]);

    expect(compile('(- 2 4 5)').forms[0]).
      to.deep.equal(['-', 2, 4, 5]);

    expect(compile('(+ (/ 2 1) (/ 9 8))').forms[0]).
      to.deep.equal(['+', ['/', 2, 1], ['/', 9, 8]]);

    expect(compile('(show 2 4)').forms[0]).
      to.deep.equal(['show', 2, 4]);

    expect(compile('(shot true 4)').forms[0]).
      to.deep.equal(['shot', '#t', 4]);

    expect(compile('(shoe \'linear)').forms[0]).
      to.deep.equal(['shoe', ['quote', 'linear']]);

    expect(compile('(slow something 4)').forms[0]).
      to.deep.equal(['slow', 'something', 4]);

    expect(compile('(how \"something\" 4)').forms[0]).
      to.deep.equal(['how', ['quote', 'something'], 4]);

    expect(compile('(go arg1: 45 arg2: 11)').forms[0]).
      to.deep.equal(['go', {arg1: 45, arg2: 11}]);

    expect(compile('(go)').forms[0]).
      to.deep.equal(['go']);

  });

  it('should compile function define statements', () => {
    expect(compile('(define (add x: 0 y: 0))').forms[0]).
      to.deep.equal(['define', ['add', {x:0, y: 0}]]);

    expect(compile('(define (add x: (+ 1 1) y: 0))').forms[0]).
      to.deep.equal(['define', ['add', {x: ['+', 1, 1], y: 0}]]);
  });
});
