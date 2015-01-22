import {describe, ddescribe, it, iit, xit, xdescribe, expect, beforeEach, async, tick} from 'test_lib/test_lib';

import {Colour, Format, cloneAs} from 'seni/Colour';

export function main() {
  describe('Colour', () => {

    it('should make a colour', () => {
      
      let c = new Colour(Format.RGB, [1.0, 1.0, 1.0, 1.0])
      expect(c.format).toEqual(Format.RGB);
      expect(c.val.length).toEqual(4);

      c = new Colour(Format.LAB, [1.0, 1.0, 1.0])
      expect(c.format).toEqual(Format.LAB);
      expect(c.val.length).toEqual(4);
    });


    it('should compare colours', () => {
      let c = new Colour(Format.RGB, [1.0, 1.0, 1.0, 1.0]);
      let d = new Colour(Format.RGB, [1.0, 1.0, 1.0, 1.0]);
      let e = new Colour(Format.HSL, [1.0, 1.0, 1.0, 1.0]);
      let f = new Colour(Format.HSL, [1.0, 0.5, 1.0, 1.0]);

      expect(c.compare(d)).toEqual(true);
      expect(c.compare(e)).toEqual(false);
      expect(e.compare(f)).toEqual(false);
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
      let rgb = new Colour(Format.RGB, [0.2, 0.1, 0.5, 1.0]);
      let hsl = new Colour(Format.HSL, [255.0, 0.6666, 0.3, 1.0]);
      let lab = new Colour(Format.LAB, [19.9072, 39.6375, -52.7720, 1.0]);

      compCol(rgb.cloneAs(Format.RGB), rgb);
      compCol(rgb.cloneAs(Format.HSL), hsl);
      compCol(rgb.cloneAs(Format.LAB), lab);

      compCol(hsl.cloneAs(Format.RGB), rgb);
      compCol(hsl.cloneAs(Format.HSL), hsl);
      compCol(hsl.cloneAs(Format.LAB), lab);

      compCol(lab.cloneAs(Format.RGB), rgb);
      compCol(lab.cloneAs(Format.HSL), hsl);
      compCol(lab.cloneAs(Format.LAB), lab);
    });
  });
}

