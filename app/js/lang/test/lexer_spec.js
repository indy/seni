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
  consumeLabel,
  consumeQuoteAbbreviation,
  consumeComment
} from 'lang/lexer';

import {
  Token,
  TokenType
} from 'lang/token';

export function main() {
  describe('Lexer', () => {

    it('should error handle', () => {
      // '|' is a character that currently isn't recognised by seni

      let q = tokenise("|");
      expect(q.length).toEqual(1);
      expect(q[0].type).toEqual(TokenType.UNKNOWN);
      expect(q[0].value).toEqual("|");

      // if an illegal character is found in any part of the input, only that
      // character will be returned as the result of the lexing operation
      //
      q = tokenise("(foo bar baz) | ");
      expect(q.length).toEqual(1);
      expect(q[0].type).toEqual(TokenType.UNKNOWN);
      expect(q[0].value).toEqual("|");
    });

    it('should tokenise strings', () => {
      let q = tokenise("(go 42 3.14)");
      expect(q.length).toEqual(5);
      expect(q[0].type).toEqual(TokenType.LIST_START);
      expect(q[1].type).toEqual(TokenType.NAME);
      expect(q[2].type).toEqual(TokenType.INT);
      expect(q[3].type).toEqual(TokenType.FLOAT);
      expect(q[4].type).toEqual(TokenType.LIST_END);

      q = tokenise("(go [\"hi\"] 'SOMETHING)");
      expect(q.length).toEqual(8);
      expect(q[0].type).toEqual(TokenType.LIST_START);
      expect(q[1].type).toEqual(TokenType.NAME);
      expect(q[2].type).toEqual(TokenType.BRACKET_START);
      expect(q[3].type).toEqual(TokenType.STRING);
      expect(q[4].type).toEqual(TokenType.BRACKET_END);
      expect(q[5].type).toEqual(TokenType.QUOTE_ABBREVIATION);
      expect(q[6].type).toEqual(TokenType.NAME);
      expect(q[7].type).toEqual(TokenType.LIST_END);
    });

    it('should tokenise labeled function invocations', () => {
      let q = tokenise("(go arg1: 42)");
      expect(q.length).toEqual(5);
      expect(q[0].type).toEqual(TokenType.LIST_START);
      expect(q[1].type).toEqual(TokenType.NAME);
      expect(q[2].type).toEqual(TokenType.LABEL);
      expect(q[3].type).toEqual(TokenType.INT);
      expect(q[4].type).toEqual(TokenType.LIST_END);
    });

    it('should recognise comments', () => {
      let q = tokenise(";(go arg1: 42)");
      expect(q.length).toEqual(1);
      expect(q[0].type).toEqual(TokenType.COMMENT);

      q = tokenise(";(go arg1: 42)\n(go arg1: 42)");
      expect(q.length).toEqual(6);
      expect(q[0].type).toEqual(TokenType.COMMENT);
      expect(q[1].type).toEqual(TokenType.LIST_START);
      expect(q[2].type).toEqual(TokenType.NAME);
      expect(q[3].type).toEqual(TokenType.LABEL);
      expect(q[4].type).toEqual(TokenType.INT);
      expect(q[5].type).toEqual(TokenType.LIST_END);
    });

    it('should skip whitespace', () => {
      expect(skipWhitespace('hello')).toEqual("hello");
      expect(skipWhitespace('     hello')).toEqual("hello");
      expect(skipWhitespace(',     hello')).toEqual("hello");
      expect(skipWhitespace('     ,hello')).toEqual("hello");
      expect(skipWhitespace('\t\thello')).toEqual("hello");
    });

    it('should get the nextTokenType', () => {
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


    it('should consume an int', () => {
      let [token, rem] = consumeInt("999 remaining");
      expect(token.type).toEqual(TokenType.INT);
      expect(token.value).toEqual(999);
      expect(rem).toEqual(" remaining");
    });

    it('should consume a float', () => {
      let [token, rem] = consumeFloat("43.21 remaining");
      expect(token.type).toEqual(TokenType.FLOAT);
      expect(token.value).toBeCloseTo(43.21);
      expect(rem).toEqual(" remaining");
    });

    it('should consume a starting list', () => {
      let [token, rem] = consumeListStart("(remaining");
      expect(token.type).toEqual(TokenType.LIST_START);
      expect(token.value).toEqual(undefined);
      expect(rem).toEqual("remaining");
    });

    it('should consume an end list', () => {
      let [token, rem] = consumeListEnd(") remaining");
      expect(token.type).toEqual(TokenType.LIST_END);
      expect(token.value).toEqual(undefined);
      expect(rem).toEqual(" remaining");
    });

    it('should consume a starting bracket', () => {
      let [token, rem] = consumeBracketStart("[remaining");
      expect(token.type).toEqual(TokenType.BRACKET_START);
      expect(token.value).toEqual(undefined);
      expect(rem).toEqual("remaining");
    });

    it('should consume an end bracket', () => {
      let [token, rem] = consumeBracketEnd("] remaining");
      expect(token.type).toEqual(TokenType.BRACKET_END);
      expect(token.value).toEqual(undefined);
      expect(rem).toEqual(" remaining");
    });

    
    it('should consume a string', () => {
      let [token, rem] = consumeString("\"string\" this is remaining");
      expect(token.type).toEqual(TokenType.STRING);
      expect(token.value).toEqual("string");
      expect(rem).toEqual(" this is remaining");
    });

    it('should consume a name', () => {
      let [token, rem] = consumeName("NAME this is leftover");
      expect(token.type).toEqual(TokenType.NAME);
      expect(token.value).toEqual("NAME");
      expect(rem).toEqual(" this is leftover");
    });

    it('should consume a label', () => {
      let [token, rem] = consumeLabel("LABEL: this is leftover");
      expect(token.type).toEqual(TokenType.LABEL);
      expect(token.value).toEqual("LABEL");
      expect(rem).toEqual(" this is leftover");
    });

    it('should consume a comment', () => {
      let [token, rem] = consumeComment("; this is a comment");
      expect(token.type).toEqual(TokenType.COMMENT);
      expect(token.value).toEqual("; this is a comment");
      expect(rem).toEqual("");

      [token, rem] = consumeComment("; this is a comment\nyo");
      expect(token.type).toEqual(TokenType.COMMENT);
      expect(token.value).toEqual("; this is a comment");
      expect(rem).toEqual("yo");
    });

    it('should consume a quote abbreviation', () => {
      let [token, rem] = consumeQuoteAbbreviation("'QUOTEDNAME");
      expect(token.type).toEqual(TokenType.QUOTE_ABBREVIATION);
      expect(token.value).toEqual(undefined);
      expect(rem).toEqual("QUOTEDNAME");
    });

  });
}