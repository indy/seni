/*
 *  Seni
 *  Copyright (C) 2015  Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

import Immutable from 'immutable';
import MathUtil from './MathUtil';

const sUnitAngle = 360.0 / 12.0;
const sComplimentaryAngle = sUnitAngle * 6;
const sTriadAngle = sUnitAngle * 4;

const Format = {
  RGB: 0,
  HSL: 1,
  LAB: 2,
  HSV: 3,
  XYZ: 4
};

const ALPHA = 3;

const X = 0;
const Y = 1;
const Z = 2;

const R = 0;
const G = 1;

const L = 0;
const A = 1;
const B = 2;

const H = 0;
const S = 1;
const V = 2;

function format(colour) {
  return colour.get('format');
}

function element(colour, index) {
  return colour.get('elements').get(index);
}

function elementArray(colour) {
  return colour.get('elements').toArray();
}

// format is one of the Format constants, val is an array
function construct(fmt, elements) {
  let elementList = new Immutable.List(elements);
  if (elementList.size === 3) {
    elementList = elementList.push(1.0);
  }
  return new Immutable.Map({format: fmt, elements: elementList});
}

function getComponent(colour, component) {
  return colour.getIn(['elements', component]);
}

function setComponent(colour, component, value) {
  return colour.setIn(['elements', component], value);
}

function getAlpha(colour) {
  return colour.getIn(['elements', ALPHA]);
}

function setAlpha(colour, alpha) {
  return colour.setIn(['elements', ALPHA], alpha);
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

function rgbxyz(c) {
  // assumes that this is already in RGB format
  const rr = colourToAxis(element(c, R));
  const gg = colourToAxis(element(c, G));
  const bb = colourToAxis(element(c, B));

  return construct(Format.XYZ, [(rr * 0.4124) + (gg * 0.3576) + (bb * 0.1805),
                                (rr * 0.2126) + (gg * 0.7152) + (bb * 0.0722),
                                (rr * 0.0193) + (gg * 0.1192) + (bb * 0.9505),
                                element(c, ALPHA)]);
}

function axisToLABComponent(a) {
  if (a > 0.008856) {
    return Math.pow(a, 1.0 / 3.0);
  } else {
    return (7.787 * a) + (16.0 / 116.0);
  }
}

function xyzlab(c) {
  // assumes that this is already in XYZ format
  const xx = axisToLABComponent(element(c, X) / 95.047);
  const yy = axisToLABComponent(element(c, Y) / 100.000);
  const zz = axisToLABComponent(element(c, Z) / 108.883);

  return construct(Format.LAB, [(116.0 * yy) - 16.0,
                                500.0 * (xx - yy),
                                200.0 * (yy - zz),
                                element(c, ALPHA)]);
}

function axisToColour(a) {
  if (a > 0.0031308) {
    return (1.055 * Math.pow(a, 1.0 / 2.4)) - 0.055;
  } else {
    return a * 12.92;
  }
}

function xyzrgb(c) {
  const xx = element(c, X) / 100.0;
  const yy = element(c, Y) / 100.0;
  const zz = element(c, Z) / 100.0;

  const r = (xx * 3.2406) + (yy * -1.5372) + (zz * -0.4986);
  const g = (xx * -0.9689) + (yy * 1.8758) + (zz * 0.0415);
  const b = (xx * 0.0557) + (yy * -0.2040) + (zz * 1.0570);

  return construct(Format.RGB, [axisToColour(r),
                                axisToColour(g),
                                axisToColour(b),
                                element(c, ALPHA)]);
}

function maxChannel(c) {
  const hi = element(c, R) > element(c, G) ? R : G;
  return element(c, B) > element(c, hi) ? B : hi;
}

function minChannel(c) {
  const hi = element(c, R) < element(c, G) ? R : G;
  return element(c, B) < element(c, hi) ? B : hi;
}

function hue(c, maxChan, chroma) {
  if (chroma === 0.0) {
    return 0.0;        // invalid hue
  }
  switch (maxChan) {
  case R:
    return 60.0 * (((element(c, G) - element(c, B)) / chroma) % 6);
  case G:
    return 60.0 * (((element(c, B) - element(c, R)) / chroma) + 2.0);
  case B:
    return 60.0 * (((element(c, R) - element(c, G)) / chroma) + 4.0);
  }

  return 0.0;            // should never get here
}

function rgbhsl(c) {
  const minCh = minChannel(c);
  const minVal = element(c, minCh);

  const maxCh = maxChannel(c);
  const maxVal = element(c, maxCh);

  const chroma = maxVal - minVal;
  const h = hue(c, maxCh, chroma);
  const validHue = (chroma !== 0.0);

  const lightness = 0.5 * (minVal + maxVal);
  let saturation;
  if (chroma === 0.0) {
    saturation = 0.0;
  } else {
    saturation = chroma / (1.0 - Math.abs((2.0 * lightness) - 1.0));
  }

  const col = construct(Format.HSL,
                        [h, saturation, lightness, element(c, ALPHA)]);
  return col.set('validHue', validHue);
}

function rgbhsv(c) {
  const minCh = minChannel(c);
  const minVal = element(c, minCh);

  const maxCh = maxChannel(c);
  const maxVal = element(c, maxCh);

  const chroma = maxVal - minVal;
  const h = hue(c, maxCh, chroma);
  const validHue = (chroma !== 0.0);

  const value = maxVal;

  let saturation;
  if (chroma === 0.0) {
    saturation = 0.0;
  } else {
    saturation = chroma / value;
  }

  const col = construct(Format.HSV, [h, saturation, value, element(c, ALPHA)]);
  return col.set('validHue', validHue);
}

function chmrgb(c, chroma, h, m) {
  if (c.get('validHue') === undefined) {
    return construct(Format.RGB, [m, m, m, element(c, ALPHA)]);
  }

  const hprime = h / 60.0;
  const x = chroma * (1.0 - Math.abs((hprime % 2) - 1.0));
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

  return construct(Format.RGB, [r + m, g + m, b + m, element(c, ALPHA)]);
}

function hslrgb(c) {
  const h = element(c, H);
  const s = element(c, S);
  const l = element(c, 2); // L already defined for LAB ...bugger
  const chroma = (1.0 - Math.abs((2.0 * l) - 1.0)) * s;
  const m = l - (0.5 * chroma);

  const col = c.set('validHue', true);

  return chmrgb(col, chroma, h, m);
}

function labComponentToAxis(l) {
  if (Math.pow(l, 3.0) > 0.008856) {
    return Math.pow(l, 3.0);
  } else {
    return (l - (16.0 / 116.0)) / 7.787;
  }
}

function labxyz(c) {
  const refX = 95.047;
  const refY = 100.000;
  const refZ = 108.883;

  const y = (element(c, L) + 16.0) / 116.0;
  const x = (element(c, A) / 500.0) + y;
  const z = y - (element(c, B) / 200.0);

  const xx = labComponentToAxis(x);
  const yy = labComponentToAxis(y);
  const zz = labComponentToAxis(z);

  return construct(Format.XYZ, [refX * xx,
                                refY * yy,
                                refZ * zz,
                                element(c, ALPHA)]);
}

function hsvrgb(c) {
  const h = element(c, H);
  const s = element(c, S);
  const v = element(c, V);
  const chroma = v * s;
  const m = v - chroma;
  return chmrgb(c, chroma, h, m);
}

// todo: this can be cached
function cloneAs(c, newFormat) {
  switch (format(c)) {
  case Format.LAB:
    if (newFormat === Format.RGB) {
      return xyzrgb(labxyz(c));
    } else if (newFormat === Format.HSV) {
      return rgbhsv(xyzrgb(labxyz(c)));
    } else if (newFormat === Format.HSL) {
      return rgbhsl(xyzrgb(labxyz(c)));
    } else if (newFormat === Format.LAB) {
      return c;
    }
    break;
  case Format.HSV:
    if (newFormat === Format.RGB) {
      return hsvrgb(c);
    } else if (newFormat === Format.HSV) {
      return c;
    } else if (newFormat === Format.HSL) {
      return rgbhsl(hsvrgb(c));
    } else if (newFormat === Format.LAB) {
      return xyzlab(rgbxyz(hsvrgb(c)));
    }
    break;
  case Format.HSL:
    if (newFormat === Format.RGB) {
      return hslrgb(c);
    } else if (newFormat === Format.HSV) {
      return rgbhsv(hslrgb(c));
    } else if (newFormat === Format.HSL) {
      return c;
    } else if (newFormat === Format.LAB) {
      return xyzlab(rgbxyz(hslrgb(c)));
    }
    break;
  case Format.RGB:
    if (newFormat === Format.RGB) {
      return c;
    } else if (newFormat === Format.HSV) {
      return rgbhsv(c);
    } else if (newFormat === Format.HSL) {
      return rgbhsl(c);
    } else if (newFormat === Format.LAB) {
      return xyzlab(rgbxyz(c));
    }
    break;
  }
  // something has gone wrong if we get here
  return undefined;
}

function addAngleToHSL(c, delta) {
  const d = cloneAs(c, Format.HSL);

  // rotate the hue by the given delta
  return d.updateIn(['elements', H], h => (h + delta) % 360.0);
}

// Return the 2 colours either side of this that are 'ang' degrees away
function pair(c, ang) {
  const ret = [addAngleToHSL(c, -ang), addAngleToHSL(c, ang)];
  return ret;
}

// Returns the colour at the opposite end of the wheel
//
function complementary(c) {
  return addAngleToHSL(c, sComplimentaryAngle);
}

// Returns the 2 colours next to a complementary colour.
// e.g. if the input colour is at the 12 o'clock position,
// this will return the 5 o'clock and 7 o'clock colours
//
function splitComplementary(c) {
  return pair(addAngleToHSL(c, sComplimentaryAngle), sUnitAngle);
}

// Returns the adjacent colours.
// e.g. given a colour at 3 o'clock this will return the
// colours at 2 o'clock and 4 o'clock
//
function analagous(c) {
  return pair(c, sUnitAngle);
}

// Returns the 2 colours that will result in all 3 colours
// being evenly spaced around the colour wheel.
// e.g. given 12 o'clock this will return 4 o'clock and 8 o'clock
//
function triad(c) {
  return pair(c, sTriadAngle);
}

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

/* eslint-disable no-unused-vars */
function proceduralFn(a, b, c, d) {

  a = cloneAs(a, Format.RGB);
  b = cloneAs(b, Format.RGB);
  c = cloneAs(c, Format.RGB);
  d = cloneAs(d, Format.RGB);

  const ar = getComponent(a, R);
  const ag = getComponent(a, G);
  const ab = getComponent(a, B);

  const br = getComponent(b, R);
  const bg = getComponent(b, G);
  const bb = getComponent(b, B);

  const cr = getComponent(c, R);
  const cg = getComponent(c, G);
  const cb = getComponent(c, B);

  const dr = getComponent(d, R);
  const dg = getComponent(d, G);
  const db = getComponent(d, B);

  return function(params) {
    const t = params.t || 1.0;

    const red = ar + ab * Math.cos(MathUtil.twoPI * (cr * t + dr));
    const green = ag + ab * Math.cos(MathUtil.twoPI * (cg * t + dg));
    const blue = ab + ab * Math.cos(MathUtil.twoPI * (cb * t + db));

    //console.log('calling proceduralFn with t value of ', t);
    //console.log('colour is ', red, green, blue);

    return construct(Format.RGB, [red, green, blue, 1.0]);
  };
}
/* eslint-enable no-unused-vars */

const Colour = {
  Format,

  ALPHA,

  R,
  G,

  X,
  Y,
  Z,

  L,
  A,
  B,

  H,
  S,
  V,

  defaultColour: construct(Format.RGB, [1.0, 0.5, 0.5, 0.5]),
  debugColour: construct(Format.RGB, [0.0, 0.8, 1.0, 1.0]),

  construct,
  format,
  element,
  elementArray,
  getComponent,
  setComponent,
  getAlpha,
  setAlpha,
  cloneAs,
  complementary,
  splitComplementary,
  analagous,
  triad,

  proceduralFn
};

export default Colour;
