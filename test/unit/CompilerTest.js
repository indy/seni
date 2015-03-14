import Parser from '../../src/lang/Parser';
import Lexer from '../../src/lang/Lexer';
import Compiler from '../../src/lang/Compiler';

describe('compiler', () => {

  function simpleCompile(form) {
    // assumes that the form will compile into a single list
    let ts = Lexer.tokenise(form).tokens;
    let ast = Parser.parse(ts).nodes;
    return Compiler.compile(ast);
  }

  it('should create gensyms for bracket forms', () => {
    let res = simpleCompile('(+ 3 [4 (inRange min: 0 max: 8)])');
    expect(res.genes.length).to.equal(1);

    let gene = res.genes[0];
    expect(gene.initialValue).to.equal(4);
    expect(gene.ast).to.deep.equal(['inRange', {min: 0, max: 8}]);
    expect(gene.gensym).to.equal('__GENSYM__0__');
  });


  it('should create identity parameters ast for empty bracket forms', () => {
    let res = simpleCompile('(+ 4 [9])');
    expect(res.genes.length).to.equal(1);

    let gene = res.genes[0];
    expect(gene.initialValue).to.equal(9);
    expect(gene.ast).to.deep.equal(['identity', {value: 9}]);
    expect(gene.gensym).to.equal('__GENSYM__0__');
  });

  it('should create multiple gensyms for multiple bracket forms', () => {
    let res = simpleCompile(`(/ [5 (oddNumber min: 1 max: 9)]
                              [4 (inRange min: 0 max: 8)])`);
    expect(res.genes.length).to.equal(2);

    let gene = res.genes[0];
    expect(gene.ast).to.deep.equal(['oddNumber', {min: 1, max: 9}]);
    expect(gene.gensym).to.equal('__GENSYM__0__');

    gene = res.genes[1];
    expect(gene.initialValue).to.equal(4);
    expect(gene.ast).to.deep.equal(['inRange', {min: 0, max: 8}]);
    expect(gene.gensym).to.equal('__GENSYM__1__');
  });

  it('should increment gensyms across multiple forms', () => {
    let res = simpleCompile('(/ 1 [2]) (+ [3] [4])');
    expect(res.genes.length).to.equal(3);

    expect(res.genes[0].gensym).to.equal('__GENSYM__0__');
    expect(res.genes[1].gensym).to.equal('__GENSYM__1__');
    expect(res.genes[2].gensym).to.equal('__GENSYM__2__');
  });



  it('should test required functions', () => {

    expect(simpleCompile('4').forms[0]).
      to.equal(4);

    expect(simpleCompile('(* 2 4)').forms[0]).
      to.deep.equal(['*', 2, 4]);

    expect(simpleCompile('(- 2 4 5)').forms[0]).
      to.deep.equal(['-', 2, 4, 5]);

    expect(simpleCompile('(+ (/ 2 1) (/ 9 8))').forms[0]).
      to.deep.equal(['+', ['/', 2, 1], ['/', 9, 8]]);

    expect(simpleCompile('(show 2 4)').forms[0]).
      to.deep.equal(['show', 2, 4]);

    expect(simpleCompile('(shot true 4)').forms[0]).
      to.deep.equal(['shot', '#t', 4]);

    expect(simpleCompile('(shoe \'linear)').forms[0]).
      to.deep.equal(['shoe', ['quote', 'linear']]);

    expect(simpleCompile('(slow something 4)').forms[0]).
      to.deep.equal(['slow', 'something', 4]);

    expect(simpleCompile('(how \"something\" 4)').forms[0]).
      to.deep.equal(['how', ['quote', 'something'], 4]);

    expect(simpleCompile('(go arg1: 45 arg2: 11)').forms[0]).
      to.deep.equal(['go', {arg1: 45, arg2: 11}]);

    expect(simpleCompile('(go)').forms[0]).
      to.deep.equal(['go']);

  });

  it('should compile function define statements', () => {
    expect(simpleCompile('(define (add x: 0 y: 0))').forms[0]).
      to.deep.equal(['define', ['add', {x:0, y: 0}]]);
  });
});
