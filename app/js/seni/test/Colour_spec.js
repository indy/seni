import {describe, ddescribe, it, iit, xit, xdescribe, expect, beforeEach, async, tick} from 'test_lib/test_lib';

import * as Colour from 'seni/Colour';

export function main() {
  describe('Colour', () => {

    it('should make a colour', () => {
      
      let c = Colour.make(Colour.Format.RGB, [1.0, 1.0, 1.0, 1.0])
      expect(c.format).toEqual(Colour.Format.RGB);
      expect(c.val.length).toEqual(4);

      c = Colour.make(Colour.Format.LAB, [1.0, 1.0, 1.0])
      expect(c.format).toEqual(Colour.Format.LAB);
      expect(c.val.length).toEqual(4);
    });


    it('should compare colours', () => {
      let c = Colour.make(Colour.Format.RGB, [1.0, 1.0, 1.0, 1.0]);
      let d = Colour.make(Colour.Format.RGB, [1.0, 1.0, 1.0, 1.0]);
      let e = Colour.make(Colour.Format.HSL, [1.0, 1.0, 1.0, 1.0]);
      let f = Colour.make(Colour.Format.HSL, [1.0, 0.5, 1.0, 1.0]);

      expect(Colour.compare(c, d)).toEqual(true);
      expect(Colour.compare(c, e)).toEqual(false);
      expect(Colour.compare(e, f)).toEqual(false);
    });

    
    function compCol(a, b) {
      expect(a.format).toEqual(b.format);
      for(var i=0;i<4;i++) {
        expect(a.val[i]).toBeCloseTo(b.val[i], 2);
      }
    }

    function logColour(c, msg) {
      if(msg) {
        console.log("[" + msg + "] " + c.val);
      } else {
        console.log(c.val);
      }
    }

    it('should convert colours', () => {
      let rgb = Colour.make(Colour.Format.RGB, [0.2, 0.1, 0.5, 1.0]);
      let hsl = Colour.make(Colour.Format.HSL, [255.0, 0.6666, 0.3, 1.0]);
      let lab = Colour.make(Colour.Format.LAB, [19.9072, 39.6375, -52.7720, 1.0]);

      compCol(Colour.cloneAs(rgb, Colour.Format.RGB), rgb);
      compCol(Colour.cloneAs(rgb, Colour.Format.HSL), hsl);
      compCol(Colour.cloneAs(rgb, Colour.Format.LAB), lab);

      compCol(Colour.cloneAs(hsl, Colour.Format.RGB), rgb);
      compCol(Colour.cloneAs(hsl, Colour.Format.HSL), hsl);
      compCol(Colour.cloneAs(hsl, Colour.Format.LAB), lab);

      compCol(Colour.cloneAs(lab, Colour.Format.RGB), rgb);
      compCol(Colour.cloneAs(lab, Colour.Format.HSL), hsl);
      compCol(Colour.cloneAs(lab, Colour.Format.LAB), lab);
    });


  });
}

