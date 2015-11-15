/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

import Lexer from '../../src/lang/Lexer';
import TokenType from '../../src/lang/TokenType';

import chai from 'chai';
const expect = chai.expect;

describe('Lexer', () => {

  function t(expected, text) {
    let q = Lexer.tokenise(text).tokens;
    expect(q.length).to.equal(expected.length);

    expected.forEach((e, i) => {
      expect(q[i].type).to.equal(e);
    });
  }

  it('should error handle', () => {
    // if an illegal character is found in any part of the input, only that
    // character will be returned in the tokens list and the error property
    // will be set
    //
    // ('|' is a character that currently isn't recognised by seni)
    //
    let q = Lexer.tokenise('|');
    let tokens = q.tokens;
    expect(q.error).to.be.a('string');
    expect(tokens.length).to.equal(1);
    expect(tokens[0].type).to.equal(TokenType.UNKNOWN);
    expect(tokens[0].value).to.equal('|');

    q = Lexer.tokenise('(foo bar baz) | ');
    tokens = q.tokens;
    expect(q.error).to.be.a('string');
    expect(tokens.length).to.equal(1);
    expect(tokens[0].type).to.equal(TokenType.UNKNOWN);
    expect(tokens[0].value).to.equal('|');
  });

  it('should tokenise strings', () => {
    t([TokenType.LIST_START,
       TokenType.NAME,
       TokenType.WHITESPACE,
       TokenType.INT,
       TokenType.WHITESPACE,
       TokenType.FLOAT,
       TokenType.LIST_END],
      '(go 42 3.14)');
  });

  it('should tokenise strings 2', () => {
    t([TokenType.LIST_START,
       TokenType.NAME,
       TokenType.WHITESPACE,
       TokenType.BRACKET_START,
       TokenType.STRING,
       TokenType.BRACKET_END,
       TokenType.WHITESPACE,
       TokenType.QUOTE_ABBREVIATION,
       TokenType.NAME,
       TokenType.LIST_END],
      '(go [\"hi\"] \'SOMETHING)');
  });

  it('should tokenise labeled function invocations', () => {
    t([TokenType.LIST_START,
       TokenType.NAME,
       TokenType.WHITESPACE,
       TokenType.LABEL,
       TokenType.WHITESPACE,
       TokenType.INT,
       TokenType.LIST_END],
      '(go arg1: 42)');
  });

  it('should recognise comments', () => {
    t([TokenType.COMMENT],
      ';(go arg1: 42)');

    t([TokenType.COMMENT,
       TokenType.LIST_START,
       TokenType.NAME,
       TokenType.WHITESPACE,
       TokenType.LABEL,
       TokenType.WHITESPACE,
       TokenType.INT,
       TokenType.LIST_END],
      ';(go arg1: 42)\n(go arg1: 42)');
  });
});
