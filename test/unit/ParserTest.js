import Lexer from '../../src/lang/Lexer';
import Parser from '../../src/lang/Parser';
import NodeType from '../../src/lang/NodeType';
import Token from '../../src/lang/Token';
import TokenType from '../../src/lang/TokenType';

describe('parse', () => {

  function simpleParse(form) {
    // assumes that the form will compile into a single list
    let ts = Lexer.tokenise(form).tokens;
    return Parser.parse(ts).nodes;
  }

  it('should parse a bracketed form', () => {
    let astArray = simpleParse('(+ 1 2 [3 (int min: 0 max: 10)])');
    expect(astArray.length).to.equal(1);

    let ast = astArray[0];
    expect(ast.children.length).to.equal(4);

    expect(ast.getChild(0).alterable).to.be.false;
    expect(ast.getChild(1).alterable).to.be.false;
    expect(ast.getChild(2).alterable).to.be.false;
    expect(ast.getChild(3).alterable).to.be.true;
  });

  it('should parse an int', () => {
    let ts = [new Token(TokenType.INT, 4)];

    let res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.INT);
    expect(res[0].value).to.equal(4);
  });

  it('should parse a float', () => {
    let ts = [new Token(TokenType.FLOAT, 3.14)];

    let res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.FLOAT);
    expect(res[0].value).to.be.closeTo(3.14, 0.01);
  });

  it('should parse a name', () => {
    let ts = [new Token(TokenType.NAME, 'cdr')];

    let res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.NAME);
    expect(res[0].value).to.equal('cdr');
  });

  it('should parse a string', () => {
    let ts = [new Token(TokenType.STRING, 'hello world')];

    let res = Parser.parse(ts).nodes;

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
    let ts = [new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END)];

    let res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(1);
    expect(res[0].type).to.equal(NodeType.LIST);
  });

  it('should error on a mismatched list (no closing pair)', () => {
    let ts = [new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 4)];

    let r = Parser.parse(ts);
    expect(r.error).to.exist;
  });

  it('should error on a mismatched list (no opening pair)', () => {
    let ts = [new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END)];

    let r = Parser.parse(ts);
    expect(r.error).to.exist;
  });

  it('should parse a quoted form', () => {
    // '(2 3 4) => (quote (2 3 4))
    let ts = [new Token(TokenType.QUOTE_ABBREVIATION),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 2),
              new Token(TokenType.INT, 3),
              new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END)];

    let res = Parser.parse(ts).nodes;
    expect(res.length).to.equal(1);

    let lst = res[0];
    expect(lst.type).to.equal(NodeType.LIST);
    expect(lst.size()).to.equal(2);

    let quote = lst.getChild(0);
    expect(quote.type).to.equal(NodeType.NAME);
    expect(quote.value).to.equal('quote');

    let quotedList = lst.getChild(1);
    expect(quotedList.type).to.equal(NodeType.LIST);
    expect(quotedList.size()).to.equal(3);
  });

  it('should parse multiple lists', () => {
    let ts = [new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 4),
              new Token(TokenType.LIST_END),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 45),
              new Token(TokenType.LIST_END),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 456),
              new Token(TokenType.LIST_END)];

    let res = Parser.parse(ts).nodes;

    expect(res.length).to.equal(3);
    expect(res[0].type).to.equal(NodeType.LIST);
    expect(res[1].type).to.equal(NodeType.LIST);
    expect(res[2].type).to.equal(NodeType.LIST);
  });

  it('should parse a bracket form', () => {
    let ts = [new Token(TokenType.BRACKET_START),
              new Token(TokenType.INT, 42),
              new Token(TokenType.LIST_START),
              new Token(TokenType.INT, 22),
              new Token(TokenType.INT, 88),
              new Token(TokenType.LIST_END),
              new Token(TokenType.BRACKET_END)];

    let r = Parser.parse(ts);
    let res = r.nodes;

    expect(res.length).to.equal(1);
    let alterableNode = res[0];
    expect(alterableNode.type).to.equal(NodeType.INT);
    expect(alterableNode.value).to.equal(42);
    expect(alterableNode.alterable).to.be.true;

    let parameterNodes = alterableNode.parameterAST;
    expect(parameterNodes.length).to.equal(1);

    let params = parameterNodes[0];
    expect(params.type).to.equal(NodeType.LIST);

  });

});
