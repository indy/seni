import Interpreter from '../../src/lang/Interpreter';
import Env from '../../src/lang/Env';
import bind from '../../src/lang/bind';
import Parser from '../../src/lang/Parser';
import Lexer from '../../src/lang/Lexer';
import Compiler from '../../src/lang/Compiler';

describe('eval', () => {

  function evalForm(env, form) {
    let ts = Lexer.tokenise(form).tokens;
    let ast = Parser.parse(ts).nodes;
    let compiled = Compiler.compile(ast);
    return Interpreter.evaluate(env, compiled[0]);
  }

  var e;
  var key;
  var val;
  let epsilon = 0.01;

  beforeEach(() => {
    e = bind(new Env(), [Interpreter.specialForms,
                         Interpreter.classicFunctions]);
    key = 'foo';
    val = 5;
    e.add(key, val);
  });

  it('should evaluate simple nodes', () => {
    let res = Interpreter.evaluate(null, 4);
    expect(res).to.equal(4);

    res = Interpreter.evaluate(null, 12.34);
    expect(res).to.be.closeTo(12.34, epsilon);

    res = Interpreter.evaluate(e, ['quote', 'some string']);
    expect(res).to.equal('some string');
  });

  it('should lookup names in the env', () => {
    let res = Interpreter.evaluate(e, key);
    expect(res).to.equal(val);
  });


  it('should test required mathematical functions', () => {
    let res = evalForm(e, '(* 2 4)');
    expect(res).to.be.closeTo(8, epsilon);

    res = evalForm(e, '(+ 2 4)');
    expect(res).to.be.closeTo(6, epsilon);

    res = evalForm(e, '(- 10 3)');
    expect(res).to.be.closeTo(7, epsilon);

    res = evalForm(e, '(- 10 3 5)');
    expect(res).to.be.closeTo(2, epsilon);

    res = evalForm(e, '(- 42)');
    expect(res).to.be.closeTo(-42, epsilon);

    res = evalForm(e, '(+ 2 foo)');
    expect(res).to.be.closeTo(7, epsilon);

    res = evalForm(e, '(+ (* 2 2) (* 3 3))');
    expect(res).to.be.closeTo(13, epsilon);

    res = evalForm(e, '(/ 90 10)');
    expect(res).to.be.closeTo(9, epsilon);

    res = evalForm(e, '(/ 90 10 3)');
    expect(res).to.be.closeTo(3, epsilon);
  });


  it('should test required comparison functions', () => {
    let res = evalForm(e, '(= 90 90)');
    expect(res).to.equal('#t');

    res = evalForm(e, '(= 90 90 90)');
    expect(res).to.equal('#t');

    res = evalForm(e, '(= 90 3)');
    expect(res).to.equal('#f');

    res = evalForm(e, '(< 54 30)');
    expect(res).to.equal('#t');

    res = evalForm(e, '(< 54 30 20)');
    expect(res).to.equal('#t');

    res = evalForm(e, '(< 54 54)');
    expect(res).to.equal('#f');

    res = evalForm(e, '(< 54 540)');
    expect(res).to.equal('#f');

    res = evalForm(e, '(> 54 30)');
    expect(res).to.equal('#f');

    res = evalForm(e, '(> 54 62 72)');
    expect(res).to.equal('#t');

    res = evalForm(e, '(> 54 54)');
    expect(res).to.equal('#f');

    res = evalForm(e, '(> 54 540)');
    expect(res).to.equal('#t');
  });

  it('should test list', () => {
    let res = evalForm(e, '(list 90 90)');
    expect(res).to.deep.equal([90, 90]);
  });

  it('should test if', () => {
    let res = evalForm(e, '(if true 2 4)');
    expect(res).to.equal(2);

    res = evalForm(e, '(if false 2 4)');
    expect(res).to.equal(4);
  });

  it('should test quote', () => {
    let res = evalForm(e, '(quote something)');
    expect(res).to.equal('something');

    res = evalForm(e, '(quote (+ 4 2))');
    expect(res).to.deep.equal(['+', 4, 2]);
  });

  it('should test define', () => {
    evalForm(e, '(define monkey 42)');
    expect(e.hasBinding('monkey')).to.be.true;
    expect(e.lookup('monkey')).to.equal(42);
  });

  it('should test define for a function', () => {
    evalForm(e, '(define addup (fn (x: 2) (+ x x)))');
    expect(e.hasBinding('addup')).to.be.true;

    evalForm(e, '(set! addupres (addup x: 5))');
    expect(e.hasBinding('addupres')).to.be.true;
    expect(e.lookup('addupres')).to.equal(10);

    evalForm(e, '(set! addupres (addup))');
    expect(e.lookup('addupres')).to.equal(4);
  });

  it('should test define for a function2', () => {
    evalForm(e, '(define (addup x: 2) (+ x x))');
    expect(e.hasBinding('addup')).to.be.true;

    evalForm(e, '(set! addupres (addup x: 5))');
    expect(e.hasBinding('addupres')).to.be.true;
    expect(e.lookup('addupres')).to.equal(10);

    evalForm(e, '(set! addupres (addup))');
    expect(e.lookup('addupres')).to.equal(4);
  });


  it('should test set!', () => {
    expect(e.hasBinding('foo')).to.be.true;
    expect(e.lookup('foo')).to.equal(5);

    evalForm(e, '(set! foo 42)');

    expect(e.hasBinding('foo')).to.be.true;
    expect(e.lookup('foo')).to.equal(42);
  });

  it('should test begin', () => {
    expect(e.hasBinding('foo')).to.be.true;
    expect(e.lookup('foo')).to.equal(5);
    let res = evalForm(e, '(begin (set! foo 1) (+ 1 1) (+ 2 2))');
    expect(e.hasBinding('foo')).to.be.true;
    expect(e.lookup('foo')).to.equal(1);

    res = evalForm(e, '(begin (+ 1 1) (set! foo 3) (+ 2 2))');
    expect(e.hasBinding('foo')).to.be.true;
    expect(e.lookup('foo')).to.equal(3);

    res = evalForm(e, '(begin (+ 1 1) (+ 2 2) (set! foo 5))');
    expect(e.hasBinding('foo')).to.be.true;
    expect(e.lookup('foo')).to.equal(5);
  });

  it('should test let', () => {
    let res = evalForm(e, '(let ((a 12) (b 24)) (+ a b foo))');

    expect(e.hasBinding('foo')).to.be.true;
    expect(e.lookup('foo')).to.equal(5);

    expect(res).to.equal(41);

    // let bindings can refer to earlier bindings
    res = evalForm(e, '(let ((a 2) (b (+ a a))) (+ a b foo))');
    expect(res).to.equal(11);

    // the body of let can contain multiple forms
    res = evalForm(e, '(let ((a 2) (b (+ a a))) (set! a 100) (+ a b foo))');
    expect(res).to.equal(109);
  });

  it('should test destructuring let', () => {
    let res = evalForm(e, '(let (((x y) (list 3 4))) (+ x y foo))');
    expect(res).to.equal(12);
  });

  it('should test fn', () => {
    // (fn (x y z) (+ x y z))
    let res = evalForm(e, '((fn (x: 0 y: 0) (+ x y)) x: 2 y: 3)');
    expect(res).to.equal(5);

    res = evalForm(e, '((fn (x: 0 y: 0) (+ x y)) x: (+ 3 2) y: 3)');
    expect(res).to.equal(8);

    // body can contain multiple forms
    res = evalForm(e, '((fn (x: 0 y: 0) (+ 1 1) (+ x y)) x: (+ 3 2) y: 3)');
    expect(res).to.equal(8);

    res = evalForm(e, '((fn (x: 0 y: 0) (+ x y)) y: 3)');
    expect(res).to.equal(3);

    res = evalForm(e, '((fn (x: 0 y: 0) (+ x y foo)) x: 2 y: 3)');
    expect(res).to.equal(10);

    res = evalForm(e, '((fn (x: 3 y: 4) (+ x y)))');
    expect(res).to.equal(7);

    // the default values may need to be eval'd
    res = evalForm(e, '((fn (x: 3 y: (+ 2 2)) (+ x y)))');
    expect(res).to.equal(7);

    res = evalForm(e, '((fn () 3))');
    expect(res).to.equal(3);
  });

  it('should test loop', () => {
    e.add('bar', 0);
    let res = evalForm(e, '(loop (a from: 0 to: 4 step: 1) (set! bar (+ bar a)))');
    expect(e.lookup('bar')).to.equal(6);

    // ''until' for <= loop ('to' for < loop)
    e.add('bar', 0);
    res = evalForm(e, '(loop (a from: 0 until: 4 step: 1) (set! bar (+ bar a)))');
    expect(e.lookup('bar')).to.equal(10);

    e.add('bar', 0);
    res = evalForm(e, '(loop (a to: 5) (set! bar (+ bar a)))');
    expect(e.lookup('bar')).to.equal(10);

    e.add('bar', 0);
    res = evalForm(e, '(loop (a to: 5 step: 2) (set! bar (+ bar a)))');
    expect(e.lookup('bar')).to.equal(6);

    // loop should eval it's arguments
    e.add('bar', 0);
    res = evalForm(e, '(let ((x 2)) (loop (a to: 5 step: x) (set! bar (+ bar a))))');
    expect(e.lookup('bar')).to.equal(6);

    // loop's body should be treated as though it is wrapped in a 'begin'
    e.add('bar', 0);
    res = evalForm(e, '(let ((x 2) (y 4)) (loop (a to: 5 step: x) (+ y y) (set! bar (+ bar a))))');
    expect(e.lookup('bar')).to.equal(6);

  });
});
