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

import MatrixStack from '../../src/seni/MatrixStack';

import chai from 'chai';
const expect = chai.expect;

describe('MatrixStack', () => {

  function matrixRowColumn(m, r, c) {
    return m[(c * 4) + r];
  }

  function expectIdentity(m) {
    for (let j = 0; j < 4; j++) {
      for (let i = 0; i < 4; i++) {
        expect(matrixRowColumn(m, i, j)).to.equal(i === j ? 1 : 0);
      }
    }
  }

  let ms;

  beforeEach(() => {
    ms = new MatrixStack();
  });

  it('constructing', () => {
    expectIdentity(ms.getHead());
  });

  it('should scale', () => {
    ms.scale(10, 20);
    const m = ms.getHead();
    expect(matrixRowColumn(m, 0, 0)).to.equal(10);
    expect(matrixRowColumn(m, 1, 1)).to.equal(20);
  });

  it('should translate', () => {
    ms.translate(30, 40);
    const m = ms.getHead();
    expect(matrixRowColumn(m, 0, 3)).to.equal(30);
    expect(matrixRowColumn(m, 1, 3)).to.equal(40);
  });
  /*
  it('should rotate', () => {
    ms.translate(20, 0);
    ms.rotate(0.5);
    let m = ms.getHead();
    // todo: write a test
  });
   */
  it('should push and pop', () => {

    ms.translate(30, 40);
    let m = ms.getHead();
    expect(matrixRowColumn(m, 0, 3)).to.equal(30);
    expect(matrixRowColumn(m, 1, 3)).to.equal(40);

    ms.pushMatrix();

    ms.scale(10, 20);
    m = ms.getHead();
    expect(matrixRowColumn(m, 0, 3)).to.equal(30);
    expect(matrixRowColumn(m, 1, 3)).to.equal(40);
    expect(matrixRowColumn(m, 0, 0)).to.equal(10);
    expect(matrixRowColumn(m, 1, 1)).to.equal(20);

    ms.popMatrix();
    m = ms.getHead();
    expect(matrixRowColumn(m, 0, 3)).to.equal(30);
    expect(matrixRowColumn(m, 1, 3)).to.equal(40);
    expect(matrixRowColumn(m, 0, 0)).to.equal(1);
    expect(matrixRowColumn(m, 1, 1)).to.equal(1);
  });
});
