import {Env, bind} from 'lang/env';
import {Node, NodeType} from 'lang/node';
import {parse} from 'lang/parser';
import {Token, TokenType} from 'lang/token';
import {tokenise} from 'lang/lexer';
import {compile} from 'lang/compiler';

export function main() {
  describe('eval', () => {

    function simpleCompile(form) {
      // assumes that the form will compile into a single list
      let ts = tokenise(form);
      let ast = parse(ts);
      let compiled = compile(ast);
      return compiled[0];
    }

    it('should ', () => {
      let form = "(go arg1: 45 arg2: 11)";
      let ts = tokenise(form);
      let ast = parse(ts);
      let compiled = compile(ast);
      expect(1).toEqual(1);
    });
       
    it('should test required functions', () => {

      expect(simpleCompile("4")).
        toEqual(4);

      expect(simpleCompile("(* 2 4)")).
        toEqual(["*", 2, 4]);

      expect(simpleCompile("(- 2 4 5)")).
        toEqual(["-", 2, 4, 5]);

      expect(simpleCompile("(+ (/ 2 1) (/ 9 8))")).
        toEqual(["+", ['/', 2, 1], ['/', 9, 8]]);

      expect(simpleCompile("(show 2 4)")).
        toEqual(["show", 2, 4]);

      expect(simpleCompile("(shot true 4)")).
        toEqual(["shot", '#t', 4]);

      expect(simpleCompile("(shoe \'linear)")).
        toEqual(["shoe", ["quote", "linear"]]);

      expect(simpleCompile("(slow something 4)")).
        toEqual(["slow", 'something', 4]);

      expect(simpleCompile("(how \"something\" 4)")).
        toEqual(["how", ["quote", 'something'], 4]);
      
      expect(simpleCompile("(go arg1: 45 arg2: 11)")).
        toEqual(["go", {arg1: 45, arg2: 11}]);
    });
  })
}
