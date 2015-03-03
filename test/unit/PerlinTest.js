import Perlin from '../../src/seni/Perlin';

describe('Perlin', () => {

  it('should output a number', () => {
    for(let i=0;i<1000;i++) {
      let binding = Perlin.perlin;
      let v = binding.create(binding)({});
      expect(v).to.be.at.least(0.0);
      expect(v).to.be.at.most(1.0);
    }
  });

  it('should output the same number given the same arguments', () => {
    let binding = Perlin.perlin;
    let v = binding.create(binding)({x: 0.1, y: 0.3, z: 0.5});
    let w = binding.create(binding)({x: 0.1, y: 0.3, z: 0.5});
    expect(v).to.be.closeTo(w, 3);
  });
});
