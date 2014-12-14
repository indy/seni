import {
    evaluate,
    bindSpecialForms,
    bindRequiredFunctions
} from '../../src/lang/interpreter';

import {
    Env
} from '../../src/lang/env';

import {
    Node,
    NodeType
} from '../../src/lang/node';

import {
  parse
} from '../../src/lang/parser';

import {
  Token,
  TokenType
} from '../../src/lang/token';

import {
  tokenise,
} from '../../src/lang/lexer';


describe('eval', function () {

    function evalForm(env, form) {
        let ts = tokenise(form);
        let ast = parse(ts);
        return evaluate(env, ast[0]);
    }

    var e;
    var key;
    var val;

    beforeEach(function () {
        e = bindSpecialForms(new Env());
        e = bindRequiredFunctions(e);

        key = "foo";
        val = new Node(NodeType.INT, 5, false);
        e.addBinding(key, val);
    });

    it('should evaluate simple nodes', function () {
        let nodeInt = new Node(NodeType.INT, 4, false);
        let res = evaluate(null, nodeInt);
        expect(res).toEqual(nodeInt);

        let nodeFloat = new Node(NodeType.FLOAT, 12.34, false);
        res = evaluate(null, nodeFloat);
        expect(res).toEqual(nodeFloat);

        let nodeBoolean = new Node(NodeType.BOOLEAN, true, false);
        res = evaluate(null, nodeBoolean);
        expect(res).toEqual(nodeBoolean);

        let nodeString = new Node(NodeType.STRING, "some string", false);
        res = evaluate(null, nodeString);
        expect(res).toEqual(nodeString);
    });

    it('should lookup names in the env', function () {
        let nodeName = new Node(NodeType.NAME, key, false);
        let res = evaluate(e, nodeName);
        expect(res).toEqual(val);
    });

    it('should test required functions', function () {
        let res = evalForm(e, "(* 2 4)");
        expect(res.getValue()).toBeCloseTo(8);

        res = evalForm(e, "(+ 2 4)");
        expect(res.getValue()).toBeCloseTo(6);

        res = evalForm(e, "(- 10 3)");
        expect(res.getValue()).toBeCloseTo(7);

        res = evalForm(e, "(- 10 3 5)");
        expect(res.getValue()).toBeCloseTo(2);

        res = evalForm(e, "(- 42)");
        expect(res.getValue()).toBeCloseTo(-42);

        res = evalForm(e, "(+ 2 foo)");
        expect(res.getValue()).toBeCloseTo(7);

        res = evalForm(e, "(+ (* 2 2) (* 3 3))");
        expect(res.getValue()).toBeCloseTo(13);

        res = evalForm(e, "(/ 90 10)");
        expect(res.getValue()).toBeCloseTo(9);

        res = evalForm(e, "(/ 90 10 3)");
        expect(res.getValue()).toBeCloseTo(3);

        res = evalForm(e, "(= 90 90)");
        expect(res.getValue()).toEqual(true);

        res = evalForm(e, "(= 90 90 90)");
        expect(res.getValue()).toEqual(true);

        res = evalForm(e, "(= 90 3)");
        expect(res.getValue()).toEqual(false);

        res = evalForm(e, "(< 54 30)");
        expect(res.getValue()).toEqual(true);

        res = evalForm(e, "(< 54 30 20)");
        expect(res.getValue()).toEqual(true);

        res = evalForm(e, "(< 54 54)");
        expect(res.getValue()).toEqual(false);

        res = evalForm(e, "(< 54 540)");
        expect(res.getValue()).toEqual(false);

        res = evalForm(e, "(> 54 30)");
        expect(res.getValue()).toEqual(false);

        res = evalForm(e, "(> 54 62 72)");
        expect(res.getValue()).toEqual(true);

        res = evalForm(e, "(> 54 54)");
        expect(res.getValue()).toEqual(false);

        res = evalForm(e, "(> 54 540)");
        expect(res.getValue()).toEqual(true);
    });

    function getAst(env, form) {
        let ts = tokenise(form);
        let ast = parse(ts);
        return ast;

    }

    it('should test if', function () {
        let res = evalForm(e, "(if true 2 4)");
        expect(res.getValue()).toEqual(2);

        res = evalForm(e, "(if false 2 4)");
        expect(res.getValue()).toEqual(4);
    });

    it('should test quote', function () {
        let res = evalForm(e, "(quote something)");
        expect(res.getType()).toEqual(NodeType.NAME);
        expect(res.getValue()).toEqual("something");

        let res = evalForm(e, "(quote (+ 4 2))");
        expect(res.getType()).toEqual(NodeType.LIST);
    });

    it('should test define', function () {
        let res = evalForm(e, "(define monkey 42)");
        expect(e.hasBinding('monkey')).toEqual(true);
        expect(e.lookup('monkey').getValue()).toEqual(42);
    });

    it('should test set!', function () {
        expect(e.hasBinding('foo')).toEqual(true);
        expect(e.lookup('foo').getValue()).toEqual(5);

        let res = evalForm(e, "(set! foo 42)");

        expect(e.hasBinding('foo')).toEqual(true);
        expect(e.lookup('foo').getValue()).toEqual(42);
    });
});
