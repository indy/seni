import {Bezier} from '../src/Bezier';

describe('Bezier', function () {
  var bezier;

  beforeEach(function () {
    bezier = new Bezier();
  });

  it('should double', function () {
    expect(bezier.doubler(3)).toEqual(6);
  });

  it('should double again', function () {
    expect(bezier.doubler(3)).toEqual(6);
  });

});
