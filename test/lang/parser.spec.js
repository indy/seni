import {
  parse
} from '../../src/lang/parser';

import {
  NodeType
} from '../../src/lang/node';

import {
  Token,
  TokenType
} from '../../src/lang/token';

describe('parse', function () {

  it('should parse an int', function () {
    let ts = [new Token(TokenType.INT, 4)];

    let res = parse(ts);

    expect(res.length).toEqual(1);
    expect(res[0].getType()).toEqual(NodeType.INT);
    expect(res[0].getValue()).toEqual(4);
  });

  it('should parse a float', function () {
    let ts = [new Token(TokenType.FLOAT, 3.14)];

    let res = parse(ts);

    expect(res.length).toEqual(1);
    expect(res[0].getType()).toEqual(NodeType.FLOAT);
    expect(res[0].getValue()).toBeCloseTo(3.14);
  });

  it('should parse a name', function () {
    let ts = [new Token(TokenType.NAME, "cdr")];

    let res = parse(ts);

    expect(res.length).toEqual(1);
    expect(res[0].getType()).toEqual(NodeType.NAME);
    expect(res[0].getValue()).toEqual("cdr");
  });

  it('should parse a string', function () {
    let ts = [new Token(TokenType.STRING, "hello world")];

    let res = parse(ts);

    expect(res.length).toEqual(1);
    expect(res[0].getType()).toEqual(NodeType.STRING);
    expect(res[0].getValue()).toEqual("hello world");
  });

  it('should parse a boolean', function () {
    let ts = [new Token(TokenType.NAME, "true")];
    let res = parse(ts);
    expect(res.length).toEqual(1);
    expect(res[0].getType()).toEqual(NodeType.BOOLEAN);
    expect(res[0].getValue()).toEqual(true);

    ts = [new Token(TokenType.NAME, "false")];
    res = parse(ts);
    expect(res.length).toEqual(1);
    expect(res[0].getType()).toEqual(NodeType.BOOLEAN);
    expect(res[0].getValue()).toEqual(false);
  });

  it('should parse a list', function () {
    let ts = [new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END)];

    let res = parse(ts);

    expect(res.length).toEqual(1);
    expect(res[0].getType()).toEqual(NodeType.LIST);
  });

  it('should parse a quoted form', function () {
    // '(2 3 4) => (quote (2 3 4))
    let ts = [new Token(TokenType.QUOTE_ABBREVIATION),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 2),
              new Token(TokenType.INT, 3),
              new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END)];

    let res = parse(ts);
    expect(res.length).toEqual(1);

    let lst = res[0];
    expect(lst.getType()).toEqual(NodeType.LIST);
    expect(lst.size()).toEqual(2);

    let quote = lst.getChild(0);
    expect(quote.getType()).toEqual(NodeType.NAME);
    expect(quote.getValue()).toEqual("quote");

    let quotedList = lst.getChild(1);
    expect(quotedList.getType()).toEqual(NodeType.LIST);
    expect(quotedList.size()).toEqual(3);
  });

  it('should parse multiple lists', function () {
    let ts = [new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 45),
              new Token(TokenType.LIST_END),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 456),
              new Token(TokenType.LIST_END)];

    let res = parse(ts);

    expect(res.length).toEqual(3);
    expect(res[0].getType()).toEqual(NodeType.LIST);
    expect(res[1].getType()).toEqual(NodeType.LIST);
    expect(res[2].getType()).toEqual(NodeType.LIST);
  });

  it('should parse a bracket form', function () {
    let ts = [new Token(TokenType.BRACKET_START),
              new Token(TokenType.INT, 42),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 22),
              new Token(TokenType.INT, 88),
              new Token(TokenType.LIST_END),
              new Token(TokenType.BRACKET_END)];

    let res = parse(ts);

    expect(res.length).toEqual(1);
    let alterableNode = res[0];
    expect(alterableNode.getType()).toEqual(NodeType.INT);
    expect(alterableNode.getValue()).toEqual(42);
    expect(alterableNode.isAlterable()).toEqual(true);

    let parameterNodes = alterableNode.getParameterNodes();
    expect(parameterNodes.length).toEqual(1);

    let params = parameterNodes[0];
    expect(params.getType()).toEqual(NodeType.LIST);
    
  });

});