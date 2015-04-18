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

import Token from '../../src/lang/Token';
import TokenType from '../../src/lang/TokenType';

import chai from 'chai';
const expect = chai.expect;

describe('token', () => {

  it('should be created with a type and an optional value', () => {
    let t = new Token(TokenType.INT, 4);
    expect(t.value).to.equal(4);
    expect(t.type).to.equal(TokenType.INT);

    t = new Token(TokenType.UNKNOWN);
    expect(t.value).to.equal(undefined);
  });

  it('should get values for the constants', () => {
    expect(TokenType.UNKNOWN).to.equal(0);
  });
});
