/*
    Seni
    Copyright (C) 2015 Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

// todo: look into no-unused-expressions
/* eslint-disable no-unused-expressions */

import Lexer from '../../app/src/lang/Lexer';
import Parser from '../../app/src/lang/Parser';
import NodeType from '../../app/src/lang/NodeType';
import Token from '../../app/src/lang/Token';
import TokenType from '../../app/src/lang/TokenType';

import chai from 'chai';
const expect = chai.expect;

describe('Parser', () => {

  function simpleParse(form) {
    // assumes that the form will compile into a single list
    const ts = Lexer.tokenise(form).tokens;
    return Parser.parse(ts);
  }

  it('should parse a bracketed form', () => {
    const astObj = simpleParse('(+ 1 2 {3 (int min: 0 max: 10)})');
    expect(astObj.nodes.length).to.equal(1);

    const ast = astObj.nodes[0];
    expect(ast.children.length).to.equal(7);

    expect(ast.getChild(0).alterable).to.be.false;
    expect(ast.getChild(1).alterable).to.be.false;
    expect(ast.getChild(2).alterable).to.be.false;
    expect(ast.getChild(3).alterable).to.be.false;
    expect(ast.getChild(4).alterable).to.be.false;
    expect(ast.getChild(5).alterable).to.be.false;
    expect(ast.getChild(6).alterable).to.be.true;

    const alterable = ast.getChild(6);
    expect(alterable.value).to.equal(3);
    expect(alterable.parameterPrefix.length).to.equal(0);
  });

  it('should parse a bracketed form that starts with whitespace', () => {
    const astObj = simpleParse('(+ 1 2 { 3 (int min: 0 max: 10)})');
    expect(astObj.nodes.length).to.equal(1);

    const ast = astObj.nodes[0];
    expect(ast.children.length).to.equal(7);

    expect(ast.getChild(0).alterable).to.be.false;
    expect(ast.getChild(1).alterable).to.be.false;
    expect(ast.getChild(2).alterable).to.be.false;
    expect(ast.getChild(3).alterable).to.be.false;
    expect(ast.getChild(4).alterable).to.be.false;
    expect(ast.getChild(5).alterable).to.be.false;
    expect(ast.getChild(6).alterable).to.be.true;

    const alterable = ast.getChild(6);
    expect(alterable.value).to.equal(3);
    // the whitespace after the opening curly bracket
    expect(alterable.parameterPrefix.length).to.equal(1);
  });

  it('should parse an int', () => {
    const ts = [new Token(TokenType.INT, 4)];

    const res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.INT);
    expect(res[0].value).to.equal(4);
  });

  it('should parse a float', () => {
    const ts = [new Token(TokenType.FLOAT, 3.14)];

    const res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.FLOAT);
    expect(res[0].value).to.be.closeTo(3.14, 0.01);
  });

  it('should parse a name', () => {
    const ts = [new Token(TokenType.NAME, 'cdr')];

    const res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.NAME);
    expect(res[0].value).to.equal('cdr');
  });

  it('should parse a string', () => {
    const ts = [new Token(TokenType.STRING, 'hello world')];

    const res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.STRING);
    expect(res[0].value).to.equal('hello world');
  });

  it('should parse a boolean', () => {
    let ts = [new Token(TokenType.NAME, 'true')];
    let res = Parser.parse(ts).nodes;
    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.BOOLEAN);
    expect(res[0].value).to.equal('#t');

    ts = [new Token(TokenType.NAME, 'false')];
    res = Parser.parse(ts).nodes;
    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.BOOLEAN);
    expect(res[0].value).to.equal('#f');
  });

  it('should parse a list', () => {
    const ts = [new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END)];

    const res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.LIST);
  });

  it('should error on a mismatched list (no closing pair)', () => {
    const ts = [new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 4)];

    const r = Parser.parse(ts);
    expect(r.error).to.exist;
  });

  it('should error on a mismatched list (no opening pair)', () => {
    const ts = [new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END)];

    const r = Parser.parse(ts);
    expect(r.error).to.exist;
  });

  it('should parse a quoted form', () => {
    // '(2 3 4) => (quote (2 3 4))
    const ts = [new Token(TokenType.QUOTE_ABBREVIATION),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 2),
              new Token(TokenType.INT, 3),
              new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END)];

    const res = Parser.parse(ts).nodes;
    expect(res.length).to.equal(1);

    const lst = res[0];
    expect(lst.type).to.equal(NodeType.LIST);
    expect(lst.size()).to.equal(3);

    let quote = lst.getChild(0);
    expect(quote.type).to.equal(NodeType.NAME);
    expect(quote.value).to.equal('quote');

    quote = lst.getChild(1);
    expect(quote.type).to.equal(NodeType.WHITESPACE);

    const quotedList = lst.getChild(2);
    expect(quotedList.type).to.equal(NodeType.LIST);
    expect(quotedList.size()).to.equal(3);
  });

  it('should parse multiple lists', () => {
    const ts = [new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 45),
              new Token(TokenType.LIST_END),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 456),
              new Token(TokenType.LIST_END)];

    const res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(3);
    expect(res[0].type).to.equal(NodeType.LIST);
    expect(res[1].type).to.equal(NodeType.LIST);
    expect(res[2].type).to.equal(NodeType.LIST);
  });

  it('should parse a bracket form', () => {
    const ts = [new Token(TokenType.ALTERABLE_START),
              new Token(TokenType.INT, 42),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 22),
              new Token(TokenType.INT, 88),
              new Token(TokenType.LIST_END),
              new Token(TokenType.ALTERABLE_END)];

    const r = Parser.parse(ts);
    const res = r.nodes;

    expect(res.length).to.equal(1);
    const alterableNode = res[0];
    expect(alterableNode.type).to.equal(NodeType.INT);
    expect(alterableNode.value).to.equal(42);
    expect(alterableNode.alterable).to.be.true;

    const parameterNodes = alterableNode.parameterAST;
    expect(parameterNodes.length).to.equal(1);

    const params = parameterNodes[0];
    expect(params.type).to.equal(NodeType.LIST);
  });

  it('should parse a vector', () => {
    const ts = [new Token(TokenType.VECTOR_START),
              new Token(TokenType.INT, 4),
              new Token(TokenType.VECTOR_END)];

    const res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.VECTOR);
  });

  it('should parse multiple vectors', () => {
    const ts = [new Token(TokenType.VECTOR_START),
              new Token(TokenType.INT, 4),
              new Token(TokenType.VECTOR_END),
              new Token(TokenType.VECTOR_START),
              new Token(TokenType.INT, 45),
              new Token(TokenType.VECTOR_END),
              new Token(TokenType.VECTOR_START),
              new Token(TokenType.INT, 456),
              new Token(TokenType.VECTOR_END)];

    const res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(3);
    expect(res[0].type).to.equal(NodeType.VECTOR);
    expect(res[1].type).to.equal(NodeType.VECTOR);
    expect(res[2].type).to.equal(NodeType.VECTOR);
  });

  it('should parse nested vectors', () => {
    // [4 [45 67] [456]]
    const ts = [new Token(TokenType.VECTOR_START),
              new Token(TokenType.INT, 4),
              new Token(TokenType.VECTOR_START),
              new Token(TokenType.INT, 45),
              new Token(TokenType.INT, 67),
              new Token(TokenType.VECTOR_END),
              new Token(TokenType.VECTOR_START),
              new Token(TokenType.INT, 456),
              new Token(TokenType.VECTOR_END),
              new Token(TokenType.VECTOR_END)];


    const res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.VECTOR);
    expect(res[0].size()).to.equal(3);

    expect(res[0].getChild(1).type).to.equal(NodeType.VECTOR);
    expect(res[0].getChild(1).size()).to.equal(2);
  });

});
