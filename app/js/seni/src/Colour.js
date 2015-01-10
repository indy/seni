export var ColourFormat = {
  RGB = 0,
  HSL = 1,
  LAB = 2,
  HSV = 3,
  XYZ = 4
};

/*
    public static Colour interpolate(Colour a, Colour b, float t, Format space) {

        float[] aLab = a.as(space).getVals();
        float[] bLab = b.as(space).getVals();

        return Colour.fromSpace(space,
                MathUtils.interpolate(aLab[0], bLab[0], t),
                MathUtils.interpolate(aLab[1], bLab[1], t),
                MathUtils.interpolate(aLab[2], bLab[2], t),
                MathUtils.interpolate(aLab[3], bLab[3], t));
    }

*/

export class Colour {

  constructor(format, comps) {
    this.format = format;
    // comps is an array of 4 elements
    this.val = comps;
    this.validHue = true;
  }

  getFormat() {
    return this.format;
  }

  getVals() {
    return this.val;
  }

  getValidHue() {
    return this.validHue;
  }

  setValidHue(isValid) {
    this.validHue = isValid;
  }

  // compare this to the colour c
  compare(c) {
    if(this.format !== c.getFormat()) {
      return false;
    }

    let tolerance = 0.05f;

    let cVals = c.getVals();
    for (let i = 0; i < 4; i++) {
      if (Math.abs(cVals[i] - this.val[i]) > tolerance) {
        return false;
      }
    }
    return true;
  }


  addAngleToHSL(delta) {
    // c is be a copy of this, but in HSL format
    let c = this.as(ColourFormat.HSL);

    // rotate the hue by the given delta
    c.val[H] = (c.val[H] + delta) % 360.0;

    return c;
  }

  // Return the 2 colours either side of this that are 'ang' degrees away
  pair(ang) {
    let ret = [this.addAngleToHSL(-ang), this.addAngleToHSL(ang)];
    return ret;
  }

  // Returns the colour at the opposite end of the wheel
  //
  complementary() {
    return addAngleToHSL(sComplimentaryAngle);
  }

  // Returns the 2 colours next to a complementary colour. 
  // e.g. if the input colour is at the 12 o'clock position, 
  // this will return the 5 o'clock and 7 o'clock colours
  //
  splitComplementary() {
    return this.complementary().pair(sUnitAngle);
  }

  // Returns the adjacent colours. 
  // e.g. given a colour at 3 o'clock this will return the
  // colours at 2 o'clock and 4 o'clock
  //
  analagous() {
    return this.pair(sUnitAngle);
  }

  // Returns the 2 colours that will result in all 3 colours 
  // being evenly spaced around the colour wheel. 
  // e.g. given 12 o'clock this will return 4 o'clock and 8 o'clock
  //
  triad() {
    return this.pair(sTriadAngle);
  }


  /*
    http://www.brucelindbloom.com/index.html?Equations.html

    l 0 -> 100  lightness
    a -128 -> +127   green -> red
    b -128 -> +127   cyan -> yellow
  */

  colourToAxis(c) {
    let temp;
    if (c > 0.04045) {
      temp = Math.pow((c + 0.055) / 1.055, 2.4);
    } else {
      temp = c / 12.92;
    }
    return temp * 100.0;
  }
  

  RGBToXYZ() {
    // assumes that this is already in RGB format
    let rr = this.colourToAxis(this.val[RED]);
    let gg = this.colourToAxis(this.val[GREEN]);
    let bb = this.colourToAxis(this.val[BLUE]);

    return this.fromXYZ((rr * 0.4124) + (gg * 0.3576) + (bb * 0.1805),
                        (rr * 0.2126) + (gg * 0.7152) + (bb * 0.0722),
                        (rr * 0.0193) + (gg * 0.1192) + (bb * 0.9505),
                        this.val[ALPHA]);
  }

  axisToLABComponent(a) {
    if (a > 0.008856) {
      return Math.pow(a, 1.0 / 3.0);
    } else {
      return (7.787 * a) + (16.0 / 116.0);
    }
  }
  
  XYZToLAB() {
    // assumes that this is already in XYZ format
    let xx = axisToLABComponent(this.val[X] / 95.047);
    let yy = axisToLABComponent(this.val[Y] / 100.000);
    let zz = axisToLABComponent(this.val[Z] / 108.883);

    return Colour.fromLAB((116.0 * yy) - 16.0,
                          500.0 * (xx - yy),
                          200.0 * (yy - zz),
                          this.val[ALPHA]);
  }

  AxisToColour(a) {
    if (a > 0.0031308) {
      return (1.055 * Math.pow(a, 1.0 / 2.4)) - 0.055;
    } else {
      return a * 12.92;
    }
  }

  XYZToRGB() {
    let xx = this.val[X] / 100.0;
    let yy = this.val[Y] / 100.0;
    let zz = this.val[Z] / 100.0;

    let r = (xx * 3.2406) + (yy * -1.5372) + (zz * -0.4986);
    let g = (xx * -0.9689) + (yy * 1.8758) + (zz * 0.0415);
    let b = (xx * 0.0557) + (yy * -0.2040) + (zz * 1.0570);

    return this.fromRGB(AxisToColour(r),
                        AxisToColour(g),
                        AxisToColour(b),
                        this.val[ALPHA]);
  }

  maxChannel() {
    let hi = this.val[RED] > this.val[GREEN] ? RED : GREEN;
    return this.val[BLUE] > this.val[hi] ? BLUE : hi;
  }

  minChannel() {
    let hi = this.val[RED] < this.val[GREEN] ? RED : GREEN;
    return this.val[BLUE] < this.val[hi] ? BLUE : hi;
  }

  hue(maxChan, chroma) {
    if (chroma == 0.0) {
      return 0.0;        // invalid hue
    }
    switch (maxChan) {
    case RED:
      return 60.0 * (((this.val[GREEN] - this.val[BLUE]) / chroma) % 6);
    case GREEN:
      return 60.0 * (((this.val[BLUE] - this.val[RED]) / chroma) + 2.0);
    case BLUE:
      return 60.0 * (((this.val[RED] - this.val[GREEN]) / chroma) + 4.0);
    }
    ;
    return 0.0;            // should never get here
  }

  RGBToHSL() {
    let minCh = this.minChannel();
    let minVal = this.val[minCh];

    let maxCh = this.maxChannel();
    let maxVal = this.val[maxCh];

    let chroma = maxVal - minVal;
    let h = this.hue(maxCh, chroma);
    let validHue = (chroma !== 0.0);

    let lightness = 0.5 * (minVal + maxVal);
    let saturation;
    if (chroma == 0.0) {
      saturation = 0.0;
    } else {
      saturation = chroma / (1.0 - Math.abs((2.0 * lightness) - 1.0));
    }

    let ret = new ColourHSL(h, saturation, lightness, this.val[ALPHA]);
    ret.setValidHue(validHue);
    return ret;
  }

  RGBToHSV() {
    let minCh = this.minChannel();
    let minVal = this.val[minCh];

    let maxCh = this.maxChannel();
    let maxVal = this.val[maxCh];

    let chroma = maxVal - minVal;
    let h = this.hue(maxCh, chroma);
    let validHue = (chroma !== 0.0);

    let value = maxVal;

    let saturation;
    if (chroma == 0.0) {
      saturation = 0.0;
    } else {
      saturation = chroma / value;
    }

    let ret = new ColourHSV(h, saturation, value, this.val[ALPHA]);
    ret.setValidHue(validHue);
    return ret;
  }

  CHMToRGB(chroma, h, m, validHue) {
    if (!validHue) {
      return this.fromRGB(m, m, m, this.val[ALPHA]);
    }

    let hprime = h / 60.0;
    let x = chroma * (1.0 - Math.abs((hprime % 2) - 1.0));
    let r = 0.0;
    let g = 0.0;
    let b = 0.0;

    if (hprime < 1.0) {
      r = chroma;
      g = x;
      b = 0.0;
    } else if (hprime < 2.0) {
      r = x;
      g = chroma;
      b = 0.0;
    } else if (hprime < 3.0) {
      r = 0.0;
      g = chroma;
      b = x;
    } else if (hprime < 4.0) {
      r = 0.0;
      g = x;
      b = chroma;
    } else if (hprime < 5.0) {
      r = x;
      g = 0.0;
      b = chroma;
    } else if (hprime < 6.0) {
      r = chroma;
      g = 0.0;
      b = x;
    }

    return this.fromRGB(r + m, g + m, b + m, this.val[ALPHA]);
  }

  HSLToRGB(validHue) {
    let h = this.val[H];
    let s = this.val[S];
    let l = this.val[2]; // L already defined for LAB ...bugger
    let chroma = (1.0 - Math.abs((2.0 * l) - 1.0)) * s;
    let m = l - (0.5 * chroma);
    return this.CHMToRGB(chroma, h, m, validHue);
  }


  LABComponentToAxis(l) {
    if (Math.pow(l, 3.0) > 0.008856) {
      return Math.pow(l, 3.0);
    } else {
      return (l - (16.0 / 116.0)) / 7.787;
    }
  }

  LABToXYZ() {
    let refX = 95.047;
    let refY = 100.000;
    let refZ = 108.883;

    let y = (this.val[L] + 16.0) / 116.0;
    let x = (this.val[A] / 500.0) + y;
    let z = y - (this.val[B] / 200.0);

    let xx = this.LABComponentToAxis(x);
    let yy = this.LABComponentToAxis(y);
    let zz = this.LABComponentToAxis(z);

    return this.fromXYZ(refX * xx,
                        refY * yy,
                        refZ * zz,
                        this.val[ALPHA]);
  }

  HSVToRGB(validHue) {
    let h = this.val[H];
    let s = this.val[S];
    let v = this.val[V];
    let chroma = v * s;
    let m = v - chroma;
    return this.CHMToRGB(chroma, h, m, validHue);
  }

  as(newFormat) {
    switch(this.format) {
    case ColourFormat.LAB:
      if (newFormat == ColourFormat.RGB) {
        return this.LABToXYZ().XYZToRGB();
      } else if (newFormat == ColourFormat.HSV) {
        return this.LABToXYZ().XYZToRGB().RGBToHSV();
      } else if (newFormat == ColourFormat.HSL) {
        return this.LABToXYZ().XYZToRGB().RGBToHSL();
      } else if (newFormat == ColourFormat.LAB) {
        return this.fromLAB(this.val[0], this.val[1], this.val[2], this.val[3]);
      }
      break;
    case ColourFormat.HSV:
      if (newFormat == ColourFormat.RGB) {
        return this.HSVToRGB(this.validHue);
      } else if (newFormat == ColourFormat.HSV) {
        return this.fromHSV(this.val[0], this.val[1], this.val[2], this.val[3]);
      } else if (newFormat == ColourFormat.HSL) {
        return this.HSVToRGB(this.validHue).RGBToHSL();
      } else if (newFormat == ColourFormat.LAB) {
        return this.HSVToRGB(this.validHue).RGBToXYZ().XYZToLAB();
      }
      break;
    case ColourFormat.HSL:
      if (newFormat == ColourFormat.RGB) {
        return this.HSLToRGB(this.validHue);
      } else if (newFormat == ColourFormat.HSV) {
        return this.HSLToRGB(this.validHue).RGBToHSV();
      } else if (newFormat == ColourFormat.HSL) {
        return this.fromHSL(this.val[0], this.val[1], this.val[2], this.val[3]);
      } else if (newFormat == ColourFormat.LAB) {
        return this.HSLToRGB(this.validHue).RGBToXYZ().XYZToLAB();
      }
      break;
    case ColourFormat.RGB:
      if (newFormat == ColourFormat.RGB) {
        return this.fromRGB(this.val[0], this.val[1], this.val[2], this.val[3]);
      } else if (newFormat == ColourFormat.HSV) {
        return this.RGBToHSV();
      } else if (newFormat == ColourFormat.HSL) {
        return this.RGBToHSL();
      } else if (newFormat == ColourFormat.LAB) {
        return this.RGBToXYZ().XYZToLAB();
      }
      break;
    }
    // something has gone wrong if we get here
  }

}

var sUnitAngle = 360.0 / 12.0;
var sComplimentaryAngle = sUnitAngle * 6;
var sTriadAngle = sUnitAngle * 4;

var RED = 0;
var GREEN = 1;
var BLUE = 2;
var ALPHA = 3;

var X = 0;
var Y = 1;
var Z = 2;

var L = 0;
var A = 1;
var B = 2;

var H = 0;
var S = 1;
var V = 2;

