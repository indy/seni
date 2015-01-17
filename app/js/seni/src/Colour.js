export var Format = {
  RGB: 0,
  HSL: 1,
  LAB: 2,
  HSV: 3,
  XYZ: 4
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




let sampleColour = {
  format: Format.RGB,
  val: [0.1, 1.0, 1.0, 1.0],
  validHue: true                // optional
};
*/

export function make(format, vals) {
  let col = {format: format, val: vals};
  if(vals.length === 3) {
    col.val.push(1.0);
  }
  return col;
}

export function compare(c, d) {
  
  if(c.format !== d.format) {
    return false;
  }

  let tolerance = 0.05;

  for (let i = 0; i < 4; i++) {
    if (Math.abs(d.val[i] - c.val[i]) > tolerance) {
      return false;
    }
  }
  return true;
}


function addAngleToHSL(c, delta) {
  let d = cloneAs(c, Format.HSL);

  // rotate the hue by the given delta
  d.val[H] = (d.val[H] + delta) % 360.0;

  return d;
}

// Return the 2 colours either side of this that are 'ang' degrees away
function pair(c, ang) {
  let ret = [addAngleToHSL(c, -ang), addAngleToHSL(c, ang)];
  return ret;
}

// Returns the colour at the opposite end of the wheel
//
export function complementary(c) {
  return addAngleToHSL(c, sComplimentaryAngle);
}

// Returns the 2 colours next to a complementary colour. 
// e.g. if the input colour is at the 12 o'clock position, 
// this will return the 5 o'clock and 7 o'clock colours
//
export function splitComplementary(c) {
  return pair(complementary(c), sUnitAngle);
}

// Returns the adjacent colours. 
// e.g. given a colour at 3 o'clock this will return the
// colours at 2 o'clock and 4 o'clock
//
export function analagous(c) {
  return pair(c, sUnitAngle);
}

// Returns the 2 colours that will result in all 3 colours 
// being evenly spaced around the colour wheel. 
// e.g. given 12 o'clock this will return 4 o'clock and 8 o'clock
//
export function triad(c) {
  return pair(c, sTriadAngle);
}



//  http://www.brucelindbloom.com/index.html?Equations.html

//  l 0 -> 100  lightness
//  a -128 -> +127   green -> red
//  b -128 -> +127   cyan -> yellow


function colourToAxis(component) {
  let temp;
  if (component > 0.04045) {
    temp = Math.pow((component + 0.055) / 1.055, 2.4);
  } else {
    temp = component / 12.92;
  }
  return temp * 100.0;
}


function RGBToXYZ(c) {
  // assumes that this is already in RGB format
  let rr = colourToAxis(c.val[RED]);
  let gg = colourToAxis(c.val[GREEN]);
  let bb = colourToAxis(c.val[BLUE]);

  return {format: Format.XYZ,
          val: [(rr * 0.4124) + (gg * 0.3576) + (bb * 0.1805),
                (rr * 0.2126) + (gg * 0.7152) + (bb * 0.0722),
                (rr * 0.0193) + (gg * 0.1192) + (bb * 0.9505),
                c.val[ALPHA]]};
}

function axisToLABComponent(a) {
  if (a > 0.008856) {
    return Math.pow(a, 1.0 / 3.0);
  } else {
    return (7.787 * a) + (16.0 / 116.0);
  }
}

function XYZToLAB(c) {
  // assumes that this is already in XYZ format
  let xx = axisToLABComponent(c.val[X] / 95.047);
  let yy = axisToLABComponent(c.val[Y] / 100.000);
  let zz = axisToLABComponent(c.val[Z] / 108.883);

  return {format: Format.LAB,
          val:[(116.0 * yy) - 16.0,
                500.0 * (xx - yy),
                200.0 * (yy - zz),
                c.val[ALPHA]]};
}

function AxisToColour(a) {
  if (a > 0.0031308) {
    return (1.055 * Math.pow(a, 1.0 / 2.4)) - 0.055;
  } else {
    return a * 12.92;
  }
}

function XYZToRGB(c) {
  let xx = c.val[X] / 100.0;
  let yy = c.val[Y] / 100.0;
  let zz = c.val[Z] / 100.0;

  let r = (xx * 3.2406) + (yy * -1.5372) + (zz * -0.4986);
  let g = (xx * -0.9689) + (yy * 1.8758) + (zz * 0.0415);
  let b = (xx * 0.0557) + (yy * -0.2040) + (zz * 1.0570);

  return {format: Format.RGB,
          val:[AxisToColour(r),
               AxisToColour(g),
               AxisToColour(b),
               c.val[ALPHA]]};
}

function maxChannel(c) {
  let hi = c.val[RED] > c.val[GREEN] ? RED : GREEN;
  return c.val[BLUE] > c.val[hi] ? BLUE : hi;
}

function minChannel(c) {
  let hi = c.val[RED] < c.val[GREEN] ? RED : GREEN;
  return c.val[BLUE] < c.val[hi] ? BLUE : hi;
}

function hue(c, maxChan, chroma) {
  if (chroma == 0.0) {
    return 0.0;        // invalid hue
  }
  switch (maxChan) {
  case RED:
    return 60.0 * (((c.val[GREEN] - c.val[BLUE]) / chroma) % 6);
  case GREEN:
    return 60.0 * (((c.val[BLUE] - c.val[RED]) / chroma) + 2.0);
  case BLUE:
    return 60.0 * (((c.val[RED] - c.val[GREEN]) / chroma) + 4.0);
  }
  ;
  return 0.0;            // should never get here
}

function RGBToHSL(c) {
  let minCh = minChannel(c);
  let minVal = c.val[minCh];

  let maxCh = maxChannel(c);
  let maxVal = c.val[maxCh];

  let chroma = maxVal - minVal;
  let h = hue(c, maxCh, chroma);
  let validHue = (chroma !== 0.0);

  let lightness = 0.5 * (minVal + maxVal);
  let saturation;
  if (chroma == 0.0) {
    saturation = 0.0;
  } else {
    saturation = chroma / (1.0 - Math.abs((2.0 * lightness) - 1.0));
  }

  return {format: Format.HSL,
          val: [h, saturation, lightness, c.val[ALPHA]],
          validHue: validHue};
}

function RGBToHSV(c) {
  let minCh = minChannel(c);
  let minVal = c.val[minCh];

  let maxCh = maxChannel(c);
  let maxVal = c.val[maxCh];

  let chroma = maxVal - minVal;
  let h = hue(c, maxCh, chroma);
  let validHue = (chroma !== 0.0);

  let value = maxVal;

  let saturation;
  if (chroma == 0.0) {
    saturation = 0.0;
  } else {
    saturation = chroma / value;
  }

  return {format: Format.HSV,
          val: [h, saturation, value, c.val[ALPHA]],
          validHue: validHue};
}

function CHMToRGB(c, chroma, h, m) {
  if (!c.validHue) {
    return {format: Format.RGB,
            val: [m, m, m, c.val[ALPHA]]}
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

  return {format: Format.RGB,
          val: [r + m, g + m, b + m, c.val[ALPHA]]};
}

function HSLToRGB(c) {
  let h = c.val[H];
  let s = c.val[S];
  let l = c.val[2]; // L already defined for LAB ...bugger
  let chroma = (1.0 - Math.abs((2.0 * l) - 1.0)) * s;
  let m = l - (0.5 * chroma);
  return CHMToRGB({format: c.format,
                   val: c.val,
                   validHue: true}, chroma, h, m);
}


function LABComponentToAxis(l) {
  if (Math.pow(l, 3.0) > 0.008856) {
    return Math.pow(l, 3.0);
  } else {
    return (l - (16.0 / 116.0)) / 7.787;
  }
}

function LABToXYZ(c) {
  let refX = 95.047;
  let refY = 100.000;
  let refZ = 108.883;

  let y = (c.val[L] + 16.0) / 116.0;
  let x = (c.val[A] / 500.0) + y;
  let z = y - (c.val[B] / 200.0);

  let xx = LABComponentToAxis(x);
  let yy = LABComponentToAxis(y);
  let zz = LABComponentToAxis(z);

  return {format: Format.XYZ,
          val: [refX * xx,
                refY * yy,
                refZ * zz,
                c.val[ALPHA]]}
}

function HSVToRGB(c) {
  let h = c.val[H];
  let s = c.val[S];
  let v = c.val[V];
  let chroma = v * s;
  let m = v - chroma;
  return CHMToRGB(c, chroma, h, m);
}

export function cloneAs(c, newFormat) {
  switch(c.format) {
  case Format.LAB:
    if (newFormat == Format.RGB) {
      return XYZToRGB(LABToXYZ(c));
    } else if (newFormat == Format.HSV) {
      return RGBToHSV(XYZToRGB(LABToXYZ(c)));
    } else if (newFormat == Format.HSL) {
      return RGBToHSL(XYZToRGB(LABToXYZ(c)));
    } else if (newFormat == Format.LAB) {
      return {format: Format.LAB,
              val:[c.val[0], c.val[1], c.val[2], c.val[3]]};
    }
    break;
  case Format.HSV:
    if (newFormat == Format.RGB) {
      return HSVToRGB(c);
    } else if (newFormat == Format.HSV) {
      return {format: Format.HSV,
              val:[c.val[0], c.val[1], c.val[2], c.val[3]]};
    } else if (newFormat == Format.HSL) {
      return RGBToHSL(HSVToRGB(c));
    } else if (newFormat == Format.LAB) {
      return XYZToLAB(RGBToXYZ(HSVToRGB(c)));
    }
    break;
  case Format.HSL:
    if (newFormat == Format.RGB) {
      return HSLToRGB(c);
    } else if (newFormat == Format.HSV) {
      return RGBToHSV(HSLToRGB(c));
    } else if (newFormat == Format.HSL) {
      return {format: Format.HSL,
              val:[c.val[0], c.val[1], c.val[2], c.val[3]]};
    } else if (newFormat == Format.LAB) {
      return XYZToLAB(RGBToXYZ(HSLToRGB(c)));
    }
    break;
  case Format.RGB:
    if (newFormat == Format.RGB) {
      return {format: Format.RGB,
              val:[c.val[0], c.val[1], c.val[2], c.val[3]]};
    } else if (newFormat == Format.HSV) {
      return RGBToHSV(c);
    } else if (newFormat == Format.HSL) {
      return RGBToHSL(c);
    } else if (newFormat == Format.LAB) {
      return XYZToLAB(RGBToXYZ(c));
    }
    break;
  }
  // something has gone wrong if we get here
}


var sUnitAngle = 360.0 / 12.0;
var sComplimentaryAngle = sUnitAngle * 6;
var sTriadAngle = sUnitAngle * 4;

export var RED = 0;
export var GREEN = 1;
export var BLUE = 2;
export var ALPHA = 3;

export var X = 0;
export var Y = 1;
export var Z = 2;

export var L = 0;
export var A = 1;
export var B = 2;

export var H = 0;
export var S = 1;
export var V = 2;


