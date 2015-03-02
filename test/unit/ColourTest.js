import Colour from '../../src/seni/Colour';

let Format = Colour.Format;

describe('Colour', () => {

  beforeEach(function() {
  });

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

    let c = Colour.construct(Format.RGB, [0.1, 0.2, 0.3, 0.4]);
    let d = Colour.setAlpha(c, 0.8);

    expect(Colour.format(d)).to.equal(Format.RGB);
    expect(d.get('elements').size).to.equal(4);
    expect(Colour.element(d, 0)).to.equal(0.1);
    expect(Colour.element(d, 1)).to.equal(0.2);
    expect(Colour.element(d, 2)).to.equal(0.3);
    expect(Colour.element(d, 3)).to.equal(0.8);
  });


  function compCol(a, b) {
    expect(a.format).to.equal(b.format);
    let epsilon = 0.01;

    for(var i=0;i<4;i++) {
      let aElement = Colour.element(a, i);
      let bElement = Colour.element(b, i);

      expect(aElement).to.be.closeTo(bElement, epsilon);
    }
  }

  it('should convert colours', () => {
    let rgb = Colour.construct(Format.RGB, [0.2, 0.1, 0.5, 1.0]);
    let hsl = Colour.construct(Format.HSL, [255.0, 0.6666, 0.3, 1.0]);
    let lab = Colour.construct(Format.LAB, [19.9072, 39.6375, -52.7720, 1.0]);

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
