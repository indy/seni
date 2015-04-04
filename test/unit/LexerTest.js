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

describe('Lexer', () => {

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
    let q = Lexer.tokenise('(go 42 3.14)').tokens;
    expect(q.length).to.equal(5);
    expect(q[0].type).to.equal(TokenType.LIST_START);
    expect(q[1].type).to.equal(TokenType.NAME);
    expect(q[2].type).to.equal(TokenType.INT);
    expect(q[3].type).to.equal(TokenType.FLOAT);
    expect(q[4].type).to.equal(TokenType.LIST_END);
  });

  it('should tokenise strings 2', () => {
    let q = Lexer.tokenise('(go [\"hi\"] \'SOMETHING)').tokens;
    expect(q.length).to.equal(8);
    expect(q[0].type).to.equal(TokenType.LIST_START);
    expect(q[1].type).to.equal(TokenType.NAME);
    expect(q[2].type).to.equal(TokenType.BRACKET_START);
    expect(q[3].type).to.equal(TokenType.STRING);
    expect(q[4].type).to.equal(TokenType.BRACKET_END);
    expect(q[5].type).to.equal(TokenType.QUOTE_ABBREVIATION);
    expect(q[6].type).to.equal(TokenType.NAME);
    expect(q[7].type).to.equal(TokenType.LIST_END);
  });

  it('should tokenise labeled function invocations', () => {
    let q = Lexer.tokenise('(go arg1: 42)').tokens;
    expect(q.length).to.equal(5);
    expect(q[0].type).to.equal(TokenType.LIST_START);
    expect(q[1].type).to.equal(TokenType.NAME);
    expect(q[2].type).to.equal(TokenType.LABEL);
    expect(q[3].type).to.equal(TokenType.INT);
    expect(q[4].type).to.equal(TokenType.LIST_END);
  });

  it('should recognise comments', () => {
    let q = Lexer.tokenise(';(go arg1: 42)').tokens;
    expect(q.length).to.equal(1);
    expect(q[0].type).to.equal(TokenType.COMMENT);

    q = Lexer.tokenise(';(go arg1: 42)\n(go arg1: 42)').tokens;
    expect(q.length).to.equal(6);
    expect(q[0].type).to.equal(TokenType.COMMENT);
    expect(q[1].type).to.equal(TokenType.LIST_START);
    expect(q[2].type).to.equal(TokenType.NAME);
    expect(q[3].type).to.equal(TokenType.LABEL);
    expect(q[4].type).to.equal(TokenType.INT);
    expect(q[5].type).to.equal(TokenType.LIST_END);
  });
});
