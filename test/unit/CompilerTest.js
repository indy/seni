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

    expect(simpleCompile('(define (add x: (+ 1 1) y: 0))').forms[0]).
      to.deep.equal(['define', ['add', {x: ['+', 1, 1], y: 0}]]);
  });
});
