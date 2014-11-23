import {
  stepsInclusive,
  clamp,
  distance1d,
  distance2d,
  normalize,
  remapFn
} from '../src/MathUtil';

describe('MathUtil', function () {

  it('stepsInclusive', function () {
    var expected = [0.0, 0.25, 0.50, 0.75, 1.0];
    var res = stepsInclusive(0, 1, 5);

    expect(res.length).toEqual(5);
    for(var i=0;i<5;i++) {
      expect(res[i]).toEqual(expected[i]);
    }
  });

  it('clamp', function() {
    expect(clamp(5, 0, 10)).toEqual(5);
    expect(clamp(5, 7, 10)).toEqual(7);
    expect(clamp(5, 0, 4)).toEqual(4);
  });

  it('distance2d', function() {
    expect(distance2d([0, 3], [4, 0])).toEqual(5);
  });

  it('normalize', function() {
    expect(normalize(32, 0)).toEqual([1, 0]);

    let res = normalize(81, 81);
    expect(res[0]).toBeCloseTo(0.707106, 4);
    expect(res[1]).toBeCloseTo(0.707106, 4);
  });

  it('remapFn', function() {
    let res = remapFn({from: [0, 1], to: [0, 100], clamping: false});
    expect(res(0)).toBeCloseTo(0);
    expect(res(1)).toBeCloseTo(100);
    expect(res(0.4)).toBeCloseTo(40);

    res = remapFn({from: [1, 0], to: [0, 100], clamping: true});
    expect(res(0)).toBeCloseTo(100);
    expect(res(1)).toBeCloseTo(0);
    expect(res(0.4)).toBeCloseTo(60);
    expect(res(2)).toBeCloseTo(0);
    expect(res(-7)).toBeCloseTo(100);
  });

  
});
