import MathUtil from '../../src/seni/MathUtil';

describe('MathUtil', () => {

  let epsilon = 0.01;

  it('stepsInclusive', () => {
    const expected = [0.0, 0.25, 0.50, 0.75, 1.0];
    const res = MathUtil.stepsInclusive(0, 1, 5);

    expect(res.length).to.equal(5);
    for (let i = 0; i < 5; i++) {
      expect(res[i]).to.equal(expected[i]);
    }
  });

  /*
  it('clamp', () => {
    expect(MathUtil.clamp(5, 0, 10)).to.equal(5);
    expect(MathUtil.clamp(5, 7, 10)).to.equal(7);
    expect(MathUtil.clamp(5, 0, 4)).to.equal(4);
  });
   */

  it('normalize', () => {
    expect(MathUtil.normalize(32, 0)).to.eql([1, 0]);

    let res = MathUtil.normalize(81, 81);
    expect(res[0]).to.be.closeTo(0.707106, epsilon);
    expect(res[1]).to.be.closeTo(0.707106, epsilon);
  });

  it('remapFn', () => {
    let res = MathUtil.remapFn({from: [0, 1], to: [0, 100], clamping: false});
    expect(res({val: 0})).to.be.closeTo(0, epsilon);
    expect(res({val: 1})).to.be.closeTo(100, epsilon);
    expect(res({val: 0.4})).to.be.closeTo(40, epsilon);

    res = MathUtil.remapFn({from: [1, 0], to: [0, 100], clamping: true});
    expect(res({val: 0})).to.be.closeTo(100, epsilon);
    expect(res({val: 1})).to.be.closeTo(0, epsilon);
    expect(res({val: 0.4})).to.be.closeTo(60, epsilon);
    expect(res({val: 2})).to.be.closeTo(0, epsilon);
    expect(res({val: -7})).to.be.closeTo(100, epsilon);
  });
});
