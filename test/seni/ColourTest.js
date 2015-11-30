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

import Colour from '../../src/seni/Colour';

import chai from 'chai';
const expect = chai.expect;

const Format = Colour.Format;

describe('Colour', () => {

  it('construct an immutable colour map', () => {

    let c = Colour.construct(Format.RGB, [0.1, 0.2, 0.3, 0.4]);

    expect(Colour.format(c)).to.equal(Format.RGB);
    expect(c.get('elements').size).to.equal(4);
    expect(Colour.element(c, 0)).to.equal(0.1);
    expect(Colour.element(c, 1)).to.equal(0.2);
    expect(Colour.element(c, 2)).to.equal(0.3);
    expect(Colour.element(c, 3)).to.equal(0.4);

    // create a default alpha value of 1.0
    c = Colour.construct(Format.RGB, [0.9, 0.8, 0.7]);

    expect(Colour.format(c)).to.equal(Format.RGB);
    expect(c.get('elements').size).to.equal(4);
    expect(Colour.element(c, 0)).to.equal(0.9);
    expect(Colour.element(c, 1)).to.equal(0.8);
    expect(Colour.element(c, 2)).to.equal(0.7);
    expect(Colour.element(c, 3)).to.equal(1.0);
  });

  it('should return a new colour when setting alpha', () => {

    const c = Colour.construct(Format.RGB, [0.1, 0.2, 0.3, 0.4]);
    const d = Colour.setAlpha(c, 0.8);

    expect(Colour.format(d)).to.equal(Format.RGB);
    expect(d.get('elements').size).to.equal(4);
    expect(Colour.element(d, 0)).to.equal(0.1);
    expect(Colour.element(d, 1)).to.equal(0.2);
    expect(Colour.element(d, 2)).to.equal(0.3);
    expect(Colour.element(d, 3)).to.equal(0.8);
  });

  function compCol(a, b) {
    expect(a.format).to.equal(b.format);
    const epsilon = 0.01;

    for (let i = 0; i < 4; i++) {
      const aElement = Colour.element(a, i);
      const bElement = Colour.element(b, i);

      expect(aElement).to.be.closeTo(bElement, epsilon);
    }
  }

  it('should convert colours', () => {
    const rgb = Colour.construct(Format.RGB, [0.2, 0.1, 0.5, 1.0]);
    const hsl = Colour.construct(Format.HSL, [255.0, 0.6666, 0.3, 1.0]);
    const lab = Colour.construct(Format.LAB, [19.9072, 39.6375, -52.7720, 1.0]);

    compCol(Colour.cloneAs(rgb, Format.RGB), rgb);
    compCol(Colour.cloneAs(rgb, Format.HSL), hsl);
    compCol(Colour.cloneAs(rgb, Format.LAB), lab);

    compCol(Colour.cloneAs(hsl, Format.RGB), rgb);
    compCol(Colour.cloneAs(hsl, Format.HSL), hsl);
    compCol(Colour.cloneAs(hsl, Format.LAB), lab);

    compCol(Colour.cloneAs(lab, Format.RGB), rgb);
    compCol(Colour.cloneAs(lab, Format.HSL), hsl);
    compCol(Colour.cloneAs(lab, Format.LAB), lab);
  });
});
