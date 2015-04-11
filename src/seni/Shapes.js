/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

import PublicBinding from '../lang/PublicBinding';
import MathUtil from './MathUtil';
import Colour from './Colour';
import Perlin from './Perlin';
import SeedRandom from './SeedRandom';

const Format = Colour.Format;

function normals(x1, y1, x2, y2) {
  const dx = x2 - x1;
  const dy = y2 - y1;

  return [MathUtil.normalize(-dy, dx), MathUtil.normalize(dy, -dx)];
}

function bezierPoint(a, b, c, d, t) {
  const t1 = 1 - t;

  return (a * t1 * t1 * t1) +
    (3 * b * t * t1 * t1) +
    (3 * c * t * t * t1) +
    (d * t * t * t);
}

function quadraticPoint(a, b, c, t) {
  const r = ((b - a) - 0.5 * (c - a)) / (0.5 * (0.5 - 1));
  const s = c - a - r;

  return (r * t * t) + (s * t) + a;
}

function bezierCoordinates(tVals, controlPoints) {
  const xs = tVals.map((t) => bezierPoint(controlPoints[0][0],
                                          controlPoints[1][0],
                                          controlPoints[2][0],
                                          controlPoints[3][0], t));
  const ys = tVals.map((t) => bezierPoint(controlPoints[0][1],
                                          controlPoints[1][1],
                                          controlPoints[2][1],
                                          controlPoints[3][1], t));

  return {xs, ys};
}

function quadraticCoordinates(tVals, controlPoints) {
  const xs = tVals.map((t) => quadraticPoint(controlPoints[0][0],
                                             controlPoints[1][0],
                                             controlPoints[2][0], t));
  const ys = tVals.map((t) => quadraticPoint(controlPoints[0][1],
                                             controlPoints[1][1],
                                             controlPoints[2][1], t));

  return {xs, ys};
}

/*
 function debugRect(buffer, glContainer, x, y, radius, colour) {
 const elementArray = Colour.elementArray(colour);

 for(let i=0;i<3;i++) {
 buffer.prepareToAddTriangleStrip(glContainer, 4,
 [x - radius, y - radius, 0.0]);
 buffer.addVertex([x - radius, y - radius, 0.0], elementArray);
 buffer.addVertex([x + radius, y - radius, 0.0], elementArray);
 buffer.addVertex([x - radius, y + radius, 0.0], elementArray);
 buffer.addVertex([x + radius, y + radius, 0.0], elementArray);
 }
 }
 */

function getRemapAndHalfWidthEnd(params) {
  const {
    lineWidth,
    lineWidthStart,
    lineWidthEnd,
    tStart,
    tEnd,
    lineWidthMapping
  } = params;

  let halfWidthEnd, remap;

  if (lineWidth !== undefined) {
    // user has given a constant lineWidth parameter
    halfWidthEnd = lineWidth / 2.0;
    remap = () => halfWidthEnd;
  } else {
    // use the default start and end line widths
    const halfWidthStart  = lineWidthStart / 2.0;
    halfWidthEnd = lineWidthEnd / 2.0;
    remap = MathUtil.remapFn({from: [tStart, tEnd],
                              to: [halfWidthStart, halfWidthEnd],
                              mapping: lineWidthMapping});

  }

  return {halfWidthEnd, remap};
}

function addVerticesAsStrip(args) {

  let {
    renderer,
    tVals,
    xs,
    ys,
    tessellation,
    remap,
    colour,
    halfWidthEnd
  } = args;

  const glContainer = renderer.getGLContainer();
  const buffer = renderer.getBuffer();

  const elementArray = Colour.elementArray(Colour.cloneAs(colour, Format.RGB));

  for (let i = 0; i < tVals.length - 1; i++) {
    let [[xn1, yn1], [xn2, yn2]] = normals(xs[i], ys[i], xs[i + 1], ys[i + 1]),
        i0 = xs[i],
        i1 = ys[i],
        t = tVals[i];

    if (i === 0) {
      buffer.prepareToAddTriangleStrip(glContainer, tessellation * 2,
                                       [(xn1 * remap({val: t})) + i0,
                                        (yn1 * remap({val: t})) + i1,
                                        0.0]);
    }

    buffer.addVertex([(xn1 * remap({val: t})) + i0,
                      (yn1 * remap({val: t})) + i1,
                      0.0],
                     elementArray);
    buffer.addVertex([(xn2 * remap({val: t})) + i0,
                      (yn2 * remap({val: t})) + i1,
                      0.0],
                     elementArray);
  }

  // final 2 vertices for the end point
  let i = tVals.length - 2,
      [[xn1, yn1], [xn2, yn2]] = normals(xs[i], ys[i], xs[i + 1], ys[i + 1]),
      i2 = xs[i + 1],
      i3 = ys[i + 1];

  buffer.addVertex([(xn1 * halfWidthEnd) + i2,
                    (yn1 * halfWidthEnd) + i3,
                    0.0],
                   elementArray);
  buffer.addVertex([(xn2 * halfWidthEnd) + i2,
                    (yn2 * halfWidthEnd) + i3,
                    0.0],
                   elementArray);

}

function renderCurve(publicBinding, renderer, params, coordFn) {

  const fullParams = publicBinding.mergeWithDefaults(params);
  const {
    colour,
    coords,
    tessellation,
    tStart,
    tEnd
  } = fullParams;

  const tVals = MathUtil.stepsInclusive(tStart, tEnd, tessellation);

  const {
    xs,
    ys
  } = coordFn(tVals, coords);

  const {
    halfWidthEnd,
    remap
  } = getRemapAndHalfWidthEnd(fullParams);

  addVerticesAsStrip({
    renderer,
    tVals,
    xs,
    ys,
    tessellation,
    remap,
    colour,
    halfWidthEnd
  });
}

function renderSpline(publicBinding, renderer, params) {
  renderCurve(publicBinding, renderer, params, quadraticCoordinates);
}

const splineBinding = new PublicBinding(
  'spline',
  `
  `,
  {
    tessellation: 15,
    lineWidth: undefined,
    lineWidthStart: 20,
    lineWidthEnd: 20,
    lineWidthMapping: 'slow-in-out',
    coords: [[440, 400],
             [533, 700],
             [766, 200]],
    tStart: 0,
    tEnd: 1,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderSpline(self, renderer, params);
  });

function renderBezier(publicBinding, renderer, params) {
  renderCurve(publicBinding, renderer, params, bezierCoordinates);
}

const bezierBinding = new PublicBinding(
  'bezier',
  `
  this fn adds 1
  this is a multi line comment
  woo hoo
  `,
  {
    tessellation: 15,
    lineWidth: undefined,
    lineWidthStart: 20,
    lineWidthEnd: 20,
    lineWidthMapping: 'slow-in-out',
    coords: [[440, 400],
             [533, 700],
             [766, 200],
             [900, 500]],
    tStart: 0,
    tEnd: 1,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderBezier(self, renderer, params);
  });

function renderRect(publicBinding, renderer, params) {
  const glContainer = renderer.getGLContainer();
  const buffer = renderer.getBuffer();

  let {x, y, width, height, colour} = publicBinding.mergeWithDefaults(params);

  const halfWidth = (width / 2);
  const halfHeight = (height / 2);

  const elementArray = Colour.elementArray(Colour.cloneAs(colour, Format.RGB));

  buffer.prepareToAddTriangleStrip(glContainer, 4,
                                   [x - halfWidth, y - halfHeight, 0.0]);
  buffer.addVertex([x - halfWidth, y - halfHeight, 0.0], elementArray);
  buffer.addVertex([x + halfWidth, y - halfHeight, 0.0], elementArray);
  buffer.addVertex([x - halfWidth, y + halfHeight, 0.0], elementArray);
  buffer.addVertex([x + halfWidth, y + halfHeight, 0.0], elementArray);
}

const rectBinding = new PublicBinding(
  'rect',
  `
  this fn adds 1
  `,
  {
    x: 500.0,
    y: 500.0,
    width: 200,
    height: 200,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderRect(self, renderer, params);
  });

function renderBezierTrailing(publicBinding, renderer, params) {
  const {tessellation,
         lineWidth,
         coords,
         tStart,
         tEnd,
         colour} = publicBinding.mergeWithDefaults(params);

  renderBezier(bezierBinding, renderer, {tessellation: tessellation,
                                         lineWidthStart: lineWidth,
                                         lineWidthEnd: 0.0,
                                         lineWidthMapping: 'linear',
                                         coords: coords,
                                         tStart: tStart,
                                         tEnd: tEnd,
                                         colour: colour});
}

const bezierTrailingBinding = new PublicBinding(
  'bezierTrailing',
  `
  this fn adds 1
  this is a multi line comment
  woo hoo
  `,
  {
    tessellation: 15,
    lineWidth: 20,
    coords: [[440, 400],
             [533, 700],
             [766, 200],
             [900, 500]],
    tStart: 0,
    tEnd: 1,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderBezierTrailing(self, renderer, params);
  });

function renderBezierBulging(publicBinding, renderer, params) {
  const {tessellation,
         lineWidth,
         coords,
         tStart,
         tEnd,
         colour} = publicBinding.mergeWithDefaults(params);

  const tMid = (tStart + tEnd) / 2.0;
  // tessellation should be an even number
  const newTess = tessellation & 1 ? tessellation + 1 : tessellation;

  renderBezier(bezierBinding, renderer, {tessellation: newTess / 2,
                                         lineWidthStart: 0.0,
                                         lineWidthEnd: lineWidth,
                                         lineWidthMapping: 'slow-in-out',
                                         coords: coords,
                                         tStart: tStart,
                                         tEnd: tMid,
                                         colour: colour});
  renderBezier(bezierBinding, renderer, {tessellation: newTess / 2,
                                         lineWidthStart: lineWidth,
                                         lineWidthEnd: 0.0,
                                         lineWidthMapping: 'slow-in-out',
                                         coords: coords,
                                         tStart: tMid,
                                         tEnd: tEnd,
                                         colour: colour});
}

const bezierBulgingBinding = new PublicBinding(
  'bezierBulging',
  `
  `,
  {
    tessellation: 16,
    lineWidth: 20,
    coords: [[440, 400],
             [533, 700],
             [766, 200],
             [900, 500]],
    tStart: 0,
    tEnd: 1,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderBezierBulging(self, renderer, params);
  });

function renderStrokedBezier(publicBinding, renderer, params) {
  const {
    tessellation,
    lineWidth,
    coords,
    strokeTessellation,
    strokeNoise,
    strokeLineWidthStart,
    strokeLineWidthEnd,
    colour,
    colourVolatility,
    seed
  } = publicBinding.mergeWithDefaults(params);

  let [[x1, y1], [x2, y2], [x3, y3], [x4, y4]] = coords;
  let tv = MathUtil.stepsInclusive(0.0, 1.0, tessellation + 2);

  let lab = Colour.cloneAs(colour, Colour.Format.LAB);

  /* eslint-disable no-loop-func */
  for (let i = 0; i < tessellation; i++) {

    let tvals = [tv[i + 0], tv[i + 1], tv[i + 2]];
    // get 3 points on the bezier curve
    let [xx1, xx2, xx3] =
          tvals.map((t) => bezierPoint(x1, x2, x3, x4, t));
    let [yy1, yy2, yy3] =
          tvals.map((t) => bezierPoint(y1, y2, y3, y4, t));

    let ns = strokeNoise;

    let colLabL = Colour.getComponent(lab, Colour.L) +
          (Perlin._perlin(xx1, xx1, xx1) * colourVolatility);

    renderSpline(splineBinding, renderer, {
      tessellation: strokeTessellation,
      lineWidth: lineWidth,
      lineWidthStart: strokeLineWidthStart,
      lineWidthEnd: strokeLineWidthEnd,
      colour: Colour.setComponent(lab, Colour.L, colLabL),
      coords: [
        [(xx1 + (ns * Perlin._perlin(xx1, xx1, seed))),
         (yy1 + (ns * Perlin._perlin(yy1, yy1, seed)))],

        [(xx2 + (ns * Perlin._perlin(xx2, xx1, seed))),
         (yy2 + (ns * Perlin._perlin(yy2, yy1, seed)))],

        [(xx3 + (ns * Perlin._perlin(xx3, xx1, seed))),
         (yy3 + (ns * Perlin._perlin(yy3, yy1, seed)))]
      ]
    });
  }
  /* eslint-enable no-loop-func */
}

const strokedBezierBinding = new PublicBinding(
  'strokedBezier',
  `
  tessellation: the number of basic bezier curves to use to render this bezier

  lineWidth: the perceived thickness of the final bezier curve,
  made up from multiple basic bezier curves

  coords: --

  strokeWidth: the width of each basic bezier
  strokeTessellation: the tessellation of each basic bezier
  strokeNoise: the amount of noise to add to each basic bezier curve

  `,
  {
    tessellation: 15,
    lineWidth: undefined,

    coords: [[440, 400],
             [533, 700],
             [766, 200],
             [900, 500]],

    strokeTessellation: 10,
    strokeNoise: 25,
    strokeLineWidthStart: 1.0,
    strokeLineWidthEnd: 1.0,

    colour: Colour.defaultColour,
    colourVolatility: 0,

    seed: 42
  },
  (self, renderer) => {
    return (params) => renderStrokedBezier(self, renderer, params);
  });

function renderStrokedBezierRect(publicBinding, renderer, params) {
  const {
    x,
    y,
    width,
    height,
    volatility,
    overlap,
    iterations,
    seed,
    tessellation,
    strokeTessellation,
    strokeNoise,
    colour,
    colourVolatility
  } = publicBinding.mergeWithDefaults(params);

  const xStart = x - (width / 2);
  const yStart = y - (height / 2);

  const thWidth = width / 3;
  const thHeight = height / 3;
  const vol = volatility;

  const hDelta = height / iterations;
  const hStripWidth = height / iterations;

  const vDelta = width / iterations;
  const vStripWidth = width / iterations;

  const halfAlphaCol = Colour.cloneAs(colour, Colour.Format.LAB);
  const lab = Colour.setAlpha(halfAlphaCol, Colour.getAlpha(halfAlphaCol) / 2);

  const rng = SeedRandom.buildSigned(seed);
  let i;

  for (i = iterations; i > 0; i--) {
    renderStrokedBezier(strokedBezierBinding, renderer, {
      tessellation: tessellation,
      lineWidth: overlap + hStripWidth,
      coords: [
        [(rng() * vol) + xStart + (0 * thWidth),
         ((i * hDelta) + (rng() * vol) + yStart)],
        [(rng() * vol) + xStart + (1 * thWidth),
         ((i * hDelta) + (rng() * vol) + yStart)],
        [(rng() * vol) + xStart + (2 * thWidth),
         ((i * hDelta) + (rng() * vol) + yStart)],
        [(rng() * vol) + xStart + (3 * thWidth),
         ((i * hDelta) + (rng() * vol) + yStart)]
      ],
      strokeTessellation: strokeTessellation,
      strokeNoise: strokeNoise,
      colour: lab,
      colourVolatility: colourVolatility
    });
  }

  for (i = iterations; i > 0; i--) {
    renderStrokedBezier(strokedBezierBinding, renderer, {
      tessellation: tessellation,
      lineWidth: overlap + vStripWidth,
      coords: [
        [((i * vDelta) + (rng() * vol) + xStart),
         (rng() * vol) + yStart + (0 * thHeight)],
        [((i * vDelta) + (rng() * vol) + xStart),
         (rng() * vol) + yStart + (1 * thHeight)],
        [((i * vDelta) + (rng() * vol) + xStart),
         (rng() * vol) + yStart + (2 * thHeight)],
        [((i * vDelta) + (rng() * vol) + xStart),
         (rng() * vol) + yStart + (3 * thHeight)]
      ],
      strokeTessellation: strokeTessellation,
      strokeNoise: strokeNoise,
      colour: lab,
      colourVolatility: colourVolatility
    });
  }
}

const strokedBezierRectBinding = new PublicBinding(
  'strokedBezierRect',
  `
  `,
  {
    x: 100.0,
    y: 100.0,
    width: 800,
    height: 600,

    volatility: 30,
    overlap: 0.0,

    iterations: 10,
    seed: 40,

    tessellation: 15,

    strokeTessellation: 10,
    strokeNoise: 25,

    colour: Colour.defaultColour,
    colourVolatility: 40
  },
  (self, renderer) => {
    return (params) => renderStrokedBezierRect(self, renderer, params);
  });

const Shapes = {
  rect: rectBinding,
  spline: splineBinding,
  bezier: bezierBinding,
  bezierTrailing: bezierTrailingBinding,
  bezierBulging: bezierBulgingBinding,

  strokedBezier: strokedBezierBinding,
  strokedBezierRect: strokedBezierRectBinding
};

export default Shapes;
