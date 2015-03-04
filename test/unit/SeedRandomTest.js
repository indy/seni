import SeedRandom from '../../src/seni/SeedRandom';

describe('SeedRandom', () => {

  it('should have replicable number generation', () => {
    let epsilon = 0.0001;

    let aa = SeedRandom.buildUnsigned('hello.');
    expect(aa()).to.be.closeTo(0.9282578795792454, epsilon);
    expect(aa()).to.be.closeTo(0.3752569768646784, epsilon);

    let bb = SeedRandom.buildUnsigned('hello.');
    expect(bb()).to.be.closeTo(0.9282578795792454, epsilon);
    expect(bb()).to.be.closeTo(0.3752569768646784, epsilon);
  });
});
