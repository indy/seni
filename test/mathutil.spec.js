import {MathUtil, stepsInclusive} from '../src/MathUtil';

describe('MathUtil', function () {
  var mathutil;

  beforeEach(function () {
    mathutil = new MathUtil();
  });

  it('should double', function () {
    expect(mathutil.doubler(3)).toEqual(6);
  });

  it('should double again', function () {
    expect(mathutil.doubler(3)).toEqual(6);
  });

  it('stepsInclusive', function () {
    var expected = [0.0, 0.25, 0.50, 0.75, 1.0];
    var res = stepsInclusive(0, 1, 5);

    expect(res.length).toEqual(5);
    for(var i=0;i<5;i++) {
      expect(res[i]).toEqual(expected[i]);
    }
  });

});
