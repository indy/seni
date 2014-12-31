import {
  evaluate,
  requiredFunctions,
  specialForms
} from 'lang/interpreter';

import {Env, bind} from 'lang/env';
import {parse} from 'lang/parser';
import {Token, TokenType} from 'lang/token';
import {tokenise} from 'lang/lexer';

import {compile} from 'lang/compiler'

export function main() {
  describe('eval', () => {

    function evalForm(env, form) {
      let ts = tokenise(form);
      let ast = parse(ts);
      let compiled = compile(ast);
      return evaluate(env, compiled[0]);
    }

    var e;
    var key;
    var val;

    beforeEach(() => {
      e = bind(new Env(), [specialForms,
                           requiredFunctions]);

      key = "foo";
      val = 5;
      e.addBinding(key, val);
    });

    it('should evaluate simple nodes', () => {
      let res = evaluate(null, 4);
      expect(res).toEqual(4);

      res = evaluate(null, 12.34);
      expect(res).toBeCloseTo(12.34);

      res = evaluate(e, ["quote", "some string"]);
      expect(res).toEqual("some string");
    });

    it('should lookup names in the env', () => {
      let res = evaluate(e, key);
      expect(res).toEqual(val);
    });


    it('should test required mathematical functions', () => {
      let res = evalForm(e, "(* 2 4)");
      expect(res).toBeCloseTo(8);

      res = evalForm(e, "(+ 2 4)");
      expect(res).toBeCloseTo(6);

      res = evalForm(e, "(- 10 3)");
      expect(res).toBeCloseTo(7);

      res = evalForm(e, "(- 10 3 5)");
      expect(res).toBeCloseTo(2);

      res = evalForm(e, "(- 42)");
      expect(res).toBeCloseTo(-42);

      res = evalForm(e, "(+ 2 foo)");
      expect(res).toBeCloseTo(7);

      res = evalForm(e, "(+ (* 2 2) (* 3 3))");
      expect(res).toBeCloseTo(13);

      res = evalForm(e, "(/ 90 10)");
      expect(res).toBeCloseTo(9);

      res = evalForm(e, "(/ 90 10 3)");
      expect(res).toBeCloseTo(3);
    });


    it('should test required comparison functions', () => {
      let res = evalForm(e, "(= 90 90)");
      expect(res).toEqual('#t');

      res = evalForm(e, "(= 90 90 90)");
      expect(res).toEqual('#t');

      res = evalForm(e, "(= 90 3)");
      expect(res).toEqual('#f');

      res = evalForm(e, "(< 54 30)");
      expect(res).toEqual('#t');

      res = evalForm(e, "(< 54 30 20)");
      expect(res).toEqual('#t');

      res = evalForm(e, "(< 54 54)");
      expect(res).toEqual('#f');

      res = evalForm(e, "(< 54 540)");
      expect(res).toEqual('#f');

      res = evalForm(e, "(> 54 30)");
      expect(res).toEqual('#f');

      res = evalForm(e, "(> 54 62 72)");
      expect(res).toEqual('#t');

      res = evalForm(e, "(> 54 54)");
      expect(res).toEqual('#f');

      res = evalForm(e, "(> 54 540)");
      expect(res).toEqual('#t');
    });

    it('should test list', () => {
      let res = evalForm(e, "(list 90 90)");
      expect(res).toEqual([90, 90]);
    });
    
    it('should test if', () => {
      let res = evalForm(e, "(if true 2 4)");
      expect(res).toEqual(2);

      res = evalForm(e, "(if false 2 4)");
      expect(res).toEqual(4);
    });

    it('should test quote', () => {
      let res = evalForm(e, "(quote something)");
      expect(res).toEqual("something");

      res = evalForm(e, "(quote (+ 4 2))");
      expect(res).toEqual(["+", 4, 2]);
    });
    

    it('should test define', () => {
      let res = evalForm(e, "(define monkey 42)");
      expect(e.hasBinding('monkey')).toEqual(true);
      expect(e.lookup('monkey')).toEqual(42);

    });

    it('should test set!', () => {
      expect(e.hasBinding('foo')).toEqual(true);
      expect(e.lookup('foo')).toEqual(5);

      let res = evalForm(e, "(set! foo 42)");

      expect(e.hasBinding('foo')).toEqual(true);
      expect(e.lookup('foo')).toEqual(42);
    });

    it('should test begin', () => {
      expect(e.hasBinding('foo')).toEqual(true);
      expect(e.lookup('foo')).toEqual(5);
      let res = evalForm(e, "(begin (set! foo 1) (+ 1 1) (+ 2 2))");
      expect(e.hasBinding('foo')).toEqual(true);
      expect(e.lookup('foo')).toEqual(1);

      res = evalForm(e, "(begin (+ 1 1) (set! foo 3) (+ 2 2))");
      expect(e.hasBinding('foo')).toEqual(true);
      expect(e.lookup('foo')).toEqual(3);

      res = evalForm(e, "(begin (+ 1 1) (+ 2 2) (set! foo 5))");
      expect(e.hasBinding('foo')).toEqual(true);
      expect(e.lookup('foo')).toEqual(5);
    });

    it('should test let', () => {
      let res = evalForm(e, "(let ((a 12) (b 24)) (+ a b foo))");

      expect(e.hasBinding('foo')).toEqual(true);
      expect(e.lookup('foo')).toEqual(5);

      expect(res).toEqual(41);

      res = evalForm(e, "(let ((a 2) (b (+ a a))) (+ a b foo))");
      expect(res).toEqual(11);
    });

    it('should test lambda', () => {
      // (lambda (x y z) (+ x y z))
      let res = evalForm(e, "((lambda (x: 0 y: 0) (+ x y)) x: 2 y: 3)");
      expect(res).toEqual(5);

      res = evalForm(e, "((lambda (x: 0 y: 0) (+ x y)) x: (+ 3 2) y: 3)");
      expect(res).toEqual(8);

      res = evalForm(e, "((lambda (x: 0 y: 0) (+ x y)) y: 3)");
      expect(res).toEqual(3);

      res = evalForm(e, "((lambda (x: 0 y: 0) (+ x y foo)) x: 2 y: 3)");
      expect(res).toEqual(10);
      
      res = evalForm(e, "((lambda (x: 3 y: 4) (+ x y)))");
      expect(res).toEqual(7);
    });
  });
}
