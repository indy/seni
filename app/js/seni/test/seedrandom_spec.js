import {describe, expect} from 'test_lib/test_lib';

import * as seedRandom from 'seni/seedrandom';

export function main() {
  describe('seedrandom', () => {

    it('should have replicable number generation', () => {

      let aa = seedRandom.buildPRNG('hello.');
      expect(aa()).toBeCloseTo(0.9282578795792454, 4);
      expect(aa()).toBeCloseTo(0.3752569768646784, 4);

      let bb = seedRandom.buildPRNG('hello.');
      expect(bb()).toBeCloseTo(0.9282578795792454, 4);
      expect(bb()).toBeCloseTo(0.3752569768646784, 4);

    });

  });
}
