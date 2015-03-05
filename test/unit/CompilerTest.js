import Parser from '../../src/lang/Parser';
import Lexer from '../../src/lang/Lexer';
import Compiler from '../../src/lang/Compiler';

describe('compiler', () => {

  function simpleCompile(form) {
    // assumes that the form will compile into a single list
    let ts = Lexer.tokenise(form).tokens;
    let ast = Parser.parse(ts).nodes;
    let compiled = Compiler.compile(ast);
    return compiled[0];
  }

  it('should test required functions', () => {

    expect(simpleCompile('4')).
      to.equal(4);

    expect(simpleCompile('(* 2 4)')).
      to.deep.equal(['*', 2, 4]);

    expect(simpleCompile('(- 2 4 5)')).
      to.deep.equal(['-', 2, 4, 5]);

    expect(simpleCompile('(+ (/ 2 1) (/ 9 8))')).
      to.deep.equal(['+', ['/', 2, 1], ['/', 9, 8]]);

    expect(simpleCompile('(show 2 4)')).
      to.deep.equal(['show', 2, 4]);

    expect(simpleCompile('(shot true 4)')).
      to.deep.equal(['shot', '#t', 4]);

    expect(simpleCompile('(shoe \'linear)')).
      to.deep.equal(['shoe', ['quote', 'linear']]);

    expect(simpleCompile('(slow something 4)')).
      to.deep.equal(['slow', 'something', 4]);

    expect(simpleCompile('(how \"something\" 4)')).
      to.deep.equal(['how', ['quote', 'something'], 4]);

    expect(simpleCompile('(go arg1: 45 arg2: 11)')).
      to.deep.equal(['go', {arg1: 45, arg2: 11}]);

    expect(simpleCompile('(go)')).
      to.deep.equal(['go']);

  });
});
