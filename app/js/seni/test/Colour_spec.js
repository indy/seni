//import {describe, ddescribe, it, iit, xit, xdescribe, expect, beforeEach, async, tick} from 'test_lib/test_lib';

import * as Colour from 'seni/Colour';

export function main() {
  describe('Colour', () => {

    it('make a colour', () => {
      
      let c = Colour.make(Colour.Format.RGB, [1.0, 1.0, 1.0, 1.0])
      expect(c.format).toEqual(Colour.Format.RGB);
      expect(c.val.length).toEqual(4);

      c = Colour.make(Colour.Format.LAB, [1.0, 1.0, 1.0])
      expect(c.format).toEqual(Colour.Format.LAB);
      expect(c.val.length).toEqual(4);
    });


    it('compare', () => {
      let c = Colour.make(Colour.Format.RGB, [1.0, 1.0, 1.0, 1.0]);
      let d = Colour.make(Colour.Format.RGB, [1.0, 1.0, 1.0, 1.0]);
      let e = Colour.make(Colour.Format.HSL, [1.0, 1.0, 1.0, 1.0]);
      let f = Colour.make(Colour.Format.HSL, [1.0, 0.5, 1.0, 1.0]);
      
      expect(Colour.compare(c, d)).toEqual(true);
      expect(Colour.compare(c, e)).toEqual(false);
      expect(Colour.compare(e, f)).toEqual(false);
      
//      let res = normalize(81, 81);
//      expect(res[0]).toBeCloseTo(0.707106, 4);
//      expect(res[1]).toBeCloseTo(0.707106, 4);
    });


  });
}
