import {describe, ddescribe, it, iit, xit, xdescribe, expect, beforeEach, async, tick} from 'test_lib/test_lib';

import {perlin} from 'seni/Perlin';

export function main() {
  describe('Perlin noise', () => {
    it('should output a number', () => {
      for(let i=0;i<1000;i++) {
        let v = perlin.create()({});
        expect(v).toBeGreaterThan(0.0);
        expect(v).toBeLessThan(1.0);
      }
    });

    it('should output the same number given the same arguments', () => {
      let v = perlin.create()({x: 0.1, y: 0.3, z: 0.5});
      let w = perlin.create()({x: 0.1, y: 0.3, z: 0.5});

      expect(v).toBeCloseTo(w, 3);
    });

  });
}

