import {
  Lexer
} from '../../src/lang/lexer';

import {
  Token,
  TokenType
} from '../../src/lang/token';

describe('Lexer', function () {
  var lexer;

  beforeEach(function () {
    lexer = new Lexer();
  });

  it('should tokenise strings', function() {
    let q = lexer.tokenise("(go 42 3.14)");
    expect(q.length).toEqual(5);
    expect(q[0].getType()).toEqual(TokenType.LIST_START);
    expect(q[1].getType()).toEqual(TokenType.NAME);
    expect(q[2].getType()).toEqual(TokenType.INT);
    expect(q[3].getType()).toEqual(TokenType.FLOAT);
    expect(q[4].getType()).toEqual(TokenType.LIST_END);

    q = lexer.tokenise("(go [\"hi\"] 'SOMETHING)");
    expect(q.length).toEqual(8);
    expect(q[0].getType()).toEqual(TokenType.LIST_START);
    expect(q[1].getType()).toEqual(TokenType.NAME);
    expect(q[2].getType()).toEqual(TokenType.BRACKET_START);
    expect(q[3].getType()).toEqual(TokenType.STRING);
    expect(q[4].getType()).toEqual(TokenType.BRACKET_END);
    expect(q[5].getType()).toEqual(TokenType.QUOTE_ABBREVIATION);
    expect(q[6].getType()).toEqual(TokenType.NAME);
    expect(q[7].getType()).toEqual(TokenType.LIST_END);
});

  it('should detect whitespace', function () {
    expect(lexer.isWhitespace(' ')).toBe(true);
    expect(lexer.isWhitespace('d')).not.toBe(true);
  });

  it('should skip whitespace', function () {
    expect(lexer.skipWhitespace('hello')).toEqual("hello");
    expect(lexer.skipWhitespace('     hello')).toEqual("hello");
    expect(lexer.skipWhitespace('\t\thello')).toEqual("hello");
  });

  it('should get the nextTokenType', function() {
    expect(lexer.nextTokenType("'FOO")).toEqual(TokenType.QUOTE_ABBREVIATION);
    expect(lexer.nextTokenType("(FOO")).toEqual(TokenType.LIST_START);
    expect(lexer.nextTokenType(") ")).toEqual(TokenType.LIST_END);
    expect(lexer.nextTokenType("[45")).toEqual(TokenType.BRACKET_START);
    expect(lexer.nextTokenType("]")).toEqual(TokenType.BRACKET_END);
    expect(lexer.nextTokenType("\"hello\"")).toEqual(TokenType.STRING);
    expect(lexer.nextTokenType("BAR")).toEqual(TokenType.NAME);
    expect(lexer.nextTokenType("42")).toEqual(TokenType.INT);
    expect(lexer.nextTokenType("42.0")).toEqual(TokenType.FLOAT);
    expect(lexer.nextTokenType("-42")).toEqual(TokenType.INT);
    expect(lexer.nextTokenType("-42.0")).toEqual(TokenType.FLOAT);
    expect(lexer.nextTokenType(".0123")).toEqual(TokenType.FLOAT);
  });


  it('should consume an int', function() {
    let [token, rem] = lexer.consumeInt("999 remaining");
    expect(token.getType()).toEqual(TokenType.INT);
    expect(token.getValue()).toEqual(999);
    expect(rem).toEqual(" remaining");
  });

  it('should consume a float', function() {
    let [token, rem] = lexer.consumeFloat("43.21 remaining");
    expect(token.getType()).toEqual(TokenType.FLOAT);
    expect(token.getValue()).toBeCloseTo(43.21);
    expect(rem).toEqual(" remaining");
  });

  it('should consume a starting list', function() {
    let [token, rem] = lexer.consumeListStart("(remaining");
    expect(token.getType()).toEqual(TokenType.LIST_START);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual("remaining");
  });

  it('should consume an end list', function() {
    let [token, rem] = lexer.consumeListEnd(") remaining");
    expect(token.getType()).toEqual(TokenType.LIST_END);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual(" remaining");
  });

  it('should consume a starting bracket', function() {
    let [token, rem] = lexer.consumeBracketStart("[remaining");
    expect(token.getType()).toEqual(TokenType.BRACKET_START);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual("remaining");
  });

  it('should consume an end bracket', function() {
    let [token, rem] = lexer.consumeBracketEnd("] remaining");
    expect(token.getType()).toEqual(TokenType.BRACKET_END);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual(" remaining");
  });

  
  it('should consume a string', function() {
    let [token, rem] = lexer.consumeString("\"string\" this is remaining");
    expect(token.getType()).toEqual(TokenType.STRING);
    expect(token.getValue()).toEqual("string");
    expect(rem).toEqual(" this is remaining");
  });

  it('should consume a name', function() {
    let [token, rem] = lexer.consumeName("NAME this is leftover");
    expect(token.getType()).toEqual(TokenType.NAME);
    expect(token.getValue()).toEqual("NAME");
    expect(rem).toEqual(" this is leftover");
  });

  it('should consume a quote abbreviation', function() {
    let [token, rem] = lexer.consumeQuoteAbbreviation("'QUOTEDNAME");
    expect(token.getType()).toEqual(TokenType.QUOTE_ABBREVIATION);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual("QUOTEDNAME");
  });

});
