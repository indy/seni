import {
  tokenise,
  skipWhitespace,
  nextTokenType,
  consumeInt,
  consumeFloat,
  consumeListStart,
  consumeListEnd,
  consumeBracketStart,
  consumeBracketEnd,
  consumeString,
  consumeName,
  consumeQuoteAbbreviation
} from '../../src/lang/lexer';

import {
  Token,
  TokenType
} from '../../src/lang/token';

describe('Lexer', function () {

  it('should tokenise strings', function() {
    let q = tokenise("(go 42 3.14)");
    expect(q.length).toEqual(5);
    expect(q[0].getType()).toEqual(TokenType.LIST_START);
    expect(q[1].getType()).toEqual(TokenType.NAME);
    expect(q[2].getType()).toEqual(TokenType.INT);
    expect(q[3].getType()).toEqual(TokenType.FLOAT);
    expect(q[4].getType()).toEqual(TokenType.LIST_END);

    q = tokenise("(go [\"hi\"] 'SOMETHING)");
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

  it('should skip whitespace', function () {
    expect(skipWhitespace('hello')).toEqual("hello");
    expect(skipWhitespace('     hello')).toEqual("hello");
    expect(skipWhitespace('\t\thello')).toEqual("hello");
  });

  it('should get the nextTokenType', function() {
    expect(nextTokenType("'FOO")).toEqual(TokenType.QUOTE_ABBREVIATION);
    expect(nextTokenType("(FOO")).toEqual(TokenType.LIST_START);
    expect(nextTokenType(") ")).toEqual(TokenType.LIST_END);
    expect(nextTokenType("[45")).toEqual(TokenType.BRACKET_START);
    expect(nextTokenType("]")).toEqual(TokenType.BRACKET_END);
    expect(nextTokenType("\"hello\"")).toEqual(TokenType.STRING);
    expect(nextTokenType("BAR")).toEqual(TokenType.NAME);
    expect(nextTokenType("42")).toEqual(TokenType.INT);
    expect(nextTokenType("42.0")).toEqual(TokenType.FLOAT);
    expect(nextTokenType("-42")).toEqual(TokenType.INT);
    expect(nextTokenType("-42.0")).toEqual(TokenType.FLOAT);
    expect(nextTokenType(".0123")).toEqual(TokenType.FLOAT);
  });


  it('should consume an int', function() {
    let [token, rem] = consumeInt("999 remaining");
    expect(token.getType()).toEqual(TokenType.INT);
    expect(token.getValue()).toEqual(999);
    expect(rem).toEqual(" remaining");
  });

  it('should consume a float', function() {
    let [token, rem] = consumeFloat("43.21 remaining");
    expect(token.getType()).toEqual(TokenType.FLOAT);
    expect(token.getValue()).toBeCloseTo(43.21);
    expect(rem).toEqual(" remaining");
  });

  it('should consume a starting list', function() {
    let [token, rem] = consumeListStart("(remaining");
    expect(token.getType()).toEqual(TokenType.LIST_START);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual("remaining");
  });

  it('should consume an end list', function() {
    let [token, rem] = consumeListEnd(") remaining");
    expect(token.getType()).toEqual(TokenType.LIST_END);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual(" remaining");
  });

  it('should consume a starting bracket', function() {
    let [token, rem] = consumeBracketStart("[remaining");
    expect(token.getType()).toEqual(TokenType.BRACKET_START);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual("remaining");
  });

  it('should consume an end bracket', function() {
    let [token, rem] = consumeBracketEnd("] remaining");
    expect(token.getType()).toEqual(TokenType.BRACKET_END);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual(" remaining");
  });

  
  it('should consume a string', function() {
    let [token, rem] = consumeString("\"string\" this is remaining");
    expect(token.getType()).toEqual(TokenType.STRING);
    expect(token.getValue()).toEqual("string");
    expect(rem).toEqual(" this is remaining");
  });

  it('should consume a name', function() {
    let [token, rem] = consumeName("NAME this is leftover");
    expect(token.getType()).toEqual(TokenType.NAME);
    expect(token.getValue()).toEqual("NAME");
    expect(rem).toEqual(" this is leftover");
  });

  it('should consume a quote abbreviation', function() {
    let [token, rem] = consumeQuoteAbbreviation("'QUOTEDNAME");
    expect(token.getType()).toEqual(TokenType.QUOTE_ABBREVIATION);
    expect(token.getValue()).toEqual(undefined);
    expect(rem).toEqual("QUOTEDNAME");
  });

});
