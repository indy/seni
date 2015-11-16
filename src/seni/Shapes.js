/*
 *  Seni
 *  Copyright (C) 2015 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

import PublicBinding from './PublicBinding';
import MathUtil from './MathUtil';
import Colour from './Colour';
import PseudoRandom from './PseudoRandom';

function renderSpline(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderQuadratic(fullParams);
}

const splineBinding = new PublicBinding(
  'spline',
  `
  `,
  {
    tessellation: 15,
    'line-width': undefined,
    'line-width-start': 20,
    'line-width-end': 20,
    'line-width-mapping': 'slow-in-out',
    coords: [[440, 400],
             [533, 700],
             [766, 200]],
    't-start': 0,
    't-end': 1,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderSpline(self, params, renderer);
  });

function renderBezier(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderBezier(fullParams);
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
    'line-width': undefined,
    'line-width-start': 20,
    'line-width-end': 20,
    'line-width-mapping': 'slow-in-out',
    coords: [[440, 400],
             [533, 700],
             [766, 200],
             [900, 500]],
    't-start': 0,
    't-end': 1,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderBezier(self, params, renderer);
  });


function renderLine(publicBinding, params, renderer) {
  let fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderLine(fullParams);
}

const lineBinding = new PublicBinding(
  'line',
  `
  renders a line using 'from' and 'to' parameters
  `,
  {
    from: [100, 100],
    to: [500, 500],
    width: 20,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderLine(self, params, renderer);
  });

function renderRect(publicBinding, params, renderer) {
  let fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderRect(fullParams);
}

const rectBinding = new PublicBinding(
  'rect',
  `
  renders a rectangle, centered in position with the given width, height
  `,
  {
    position: [500, 500],
    width: 200,
    height: 200,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderRect(self, params, renderer);
  });


const boxBinding = new PublicBinding(
  'box',
  `
  renders a rectangle using the given top, left, bottom, right parameters
  `,
  {
    top: 300,
    left: 100,
    bottom: 100,
    right: 300,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => {
      let fullParams = self.mergeWithDefaults(params);
      let width = fullParams.right - fullParams.left;
      let height = fullParams.top - fullParams.bottom;
      let rectParams = {
        position: [fullParams.left + (width / 2),
                   fullParams.bottom + (height / 2)],
        width: width,
        height: height,
        colour: fullParams.colour
      };
      renderRect(rectBinding, rectParams, renderer);
    };
  });

function renderCircle(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderCircle(fullParams);
}

const circleBinding = new PublicBinding(
  'circle',
  `
  `,
  {
    position: [500, 500],
    radius: undefined,
    width: 200,
    height: 200,
    tessellation: 10,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderCircle(self, params, renderer);
  });

function renderCircleSlice(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderCircleSlice(fullParams);
}

const circleSliceBinding = new PublicBinding(
  'circle-slice',
  `
  `,
  {
    position: [500, 500],
    radius: undefined,
    'angle-start': 0,
    'angle-end': 180,
    width: 200,
    height: 200,
    'inner-width': 0,
    'inner-height': 0,
    tessellation: 10,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderCircleSlice(self, params, renderer);
  });

function renderBezierTrailing(publicBinding, params, renderer) {

  let fullParams = publicBinding.mergeWithDefaults(params);

  const {tessellation,
         coords,
         tStart,
         tEnd,
         colour} = fullParams;
  const lineWidth = fullParams['line-width'];

  let bezierParams = {tessellation: tessellation,
                      'line-width-start': lineWidth,
                      'line-width-end': 0.0,
                      'line-width-mapping': 'linear',
                      coords: coords,
                      't-start': tStart,
                      't-end': tEnd,
                      colour: colour};

  renderBezier(bezierBinding, bezierParams, renderer);
}

const bezierTrailingBinding = new PublicBinding(
  'bezier-trailing',
  `
  this fn adds 1
  this is a multi line comment
  woo hoo
  `,
  {
    tessellation: 15,
    'line-width': 20,
    coords: [[440, 400],
             [533, 700],
             [766, 200],
             [900, 500]],
    't-start': 0,
    't-end': 1,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderBezierTrailing(self, params, renderer);
  });

function renderBezierBulging(publicBinding, params, renderer) {

  const fullParams = publicBinding.mergeWithDefaults(params);

  const {tessellation,
         coords,
         colour} = fullParams;
  const lineWidth = fullParams['line-width'];
  const tStart = fullParams['t-start'];
  const tEnd = fullParams['t-end'];


  const tMid = (tStart + tEnd) / 2.0;
  // tessellation should be an even number
  const newTess = tessellation & 1 ? tessellation + 1 : tessellation;

  let thinFatParams = {tessellation: newTess / 2,
                       'line-width-start': 0.0,
                       'line-width-end': lineWidth,
                       'line-width-mapping': 'slow-in-out',
                       coords: coords,
                       't-start': tStart,
                       't-end': tMid,
                       colour: colour};
  renderBezier(bezierBinding, thinFatParams, renderer);

  let fatThinParams = {tessellation: newTess / 2,
                       'line-width-start': lineWidth,
                       'line-width-end': 0.0,
                       'line-width-mapping': 'slow-in-out',
                       coords: coords,
                       't-start': tMid,
                       't-end': tEnd,
                       colour: colour};
  renderBezier(bezierBinding, fatThinParams, renderer);
}

const bezierBulgingBinding = new PublicBinding(
  'bezier-bulging',
  `
  `,
  {
    tessellation: 16,
    'line-width': 20,
    coords: [[440, 400],
             [533, 700],
             [766, 200],
             [900, 500]],
    't-start': 0,
    't-end': 1,
    colour: Colour.defaultColour
  },
  (self, renderer) => {
    return (params) => renderBezierBulging(self, params, renderer);
  });

function renderStrokedBezier(publicBinding, params, renderer) {

  const fullParams = publicBinding.mergeWithDefaults(params);

  const {
    tessellation,
    coords,
    colour,
    seed
  } = fullParams;
  const lineWidth = fullParams['line-width'];
  const strokeTessellation = fullParams['stroke-tessellation'];
  const strokeNoise = fullParams['stroke-noise'];
  const strokeLineWidthStart = fullParams['stroke-line-width-start'];
  const strokeLineWidthEnd = fullParams['stroke-line-width-end'];
  const colourVolatility = fullParams['colour-volatility'];

  let [[x1, y1], [x2, y2], [x3, y3], [x4, y4]] = coords;
  let tv = MathUtil.stepsInclusive(0.0, 1.0, tessellation + 2);

  let lab = Colour.cloneAs(colour, Colour.Format.LAB);

  /* eslint-disable no-loop-func */
  for (let i = 0; i < tessellation; i++) {

    let tvals = [tv[i + 0], tv[i + 1], tv[i + 2]];
    // get 3 points on the bezier curve
    let [xx1, xx2, xx3] =
          tvals.map(t => MathUtil.bezierPoint(x1, x2, x3, x4, t));
    let [yy1, yy2, yy3] =
          tvals.map(t => MathUtil.bezierPoint(y1, y2, y3, y4, t));

    let ns = strokeNoise;

    let colLabL = Colour.getComponent(lab, Colour.L) +
          (PseudoRandom._perlin(xx1, xx1, xx1) * colourVolatility);

    let splineParams = {
      tessellation: strokeTessellation,
      'line-width': lineWidth,
      'line-width-start': strokeLineWidthStart,
      'line-width-end': strokeLineWidthEnd,
      colour: Colour.setComponent(lab, Colour.L, colLabL),
      coords: [
        [(xx1 + (ns * PseudoRandom._perlin(xx1, xx1, seed))),
         (yy1 + (ns * PseudoRandom._perlin(yy1, yy1, seed)))],

        [(xx2 + (ns * PseudoRandom._perlin(xx2, xx1, seed))),
         (yy2 + (ns * PseudoRandom._perlin(yy2, yy1, seed)))],

        [(xx3 + (ns * PseudoRandom._perlin(xx3, xx1, seed))),
         (yy3 + (ns * PseudoRandom._perlin(yy3, yy1, seed)))]
      ]
    };

    renderSpline(splineBinding, splineParams, renderer);
  }
  /* eslint-enable no-loop-func */
}

const strokedBezierBinding = new PublicBinding(
  'stroked-bezier',
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
    'line-width': undefined,

    coords: [[440, 400],
             [533, 700],
             [766, 200],
             [900, 500]],

    'stroke-tessellation': 10,
    'stroke-noise': 25,
    'stroke-line-width-start': 1.0,
    'stroke-line-width-end': 1.0,

    colour: Colour.defaultColour,
    'colour-volatility': 0,

    seed: 42
  },
  (self, renderer) => {
    return (params) => renderStrokedBezier(self, params, renderer);
  });

function renderStrokedBezierRect(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);

  const {
    position,
    width,
    height,
    volatility,
    overlap,
    iterations,
    seed,
    tessellation,
    colour
  } = fullParams;
  const strokeTessellation = fullParams['strok-tessellation'];
  const strokeNoise = fullParams['stroke-noise'];
  const colourVolatility = fullParams['colour-volatility'];

  const [x, y] = position;

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

  const rng = PseudoRandom.buildSigned(seed);
  let i;

  for (i = iterations; i > 0; i--) {
    let hParams = {
      tessellation: tessellation,
      'line-width': overlap + hStripWidth,
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
      'stroke-tessellation': strokeTessellation,
      'stroke-noise': strokeNoise,
      colour: lab,
      'colour-volatility': colourVolatility
    };
    renderStrokedBezier(strokedBezierBinding, hParams, renderer);
  }

  for (i = iterations; i > 0; i--) {
    let vParams = {
      tessellation: tessellation,
      'line-width': overlap + vStripWidth,
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
      'stroke-tessellation': strokeTessellation,
      'stroke-noise': strokeNoise,
      colour: lab,
      'colour-volatility': colourVolatility
    };
    renderStrokedBezier(strokedBezierBinding, vParams, renderer);
  }
}

const strokedBezierRectBinding = new PublicBinding(
  'stroked-bezier-rect',
  `
  `,
  {
    position: [100, 100],
    width: 800,
    height: 600,

    volatility: 30,
    overlap: 0.0,

    iterations: 10,
    seed: 40,

    tessellation: 15,

    'stroke-tessellation': 10,
    'stroke-noise': 25,

    colour: Colour.defaultColour,
    'colour-volatility': 40
  },
  (self, renderer) => {
    return (params) => renderStrokedBezierRect(self, params, renderer);
  });


const polyBinding = new PublicBinding(
  'poly',
  `renders triangle strip in which each vertex has a different colour
  `,
  {
    coords: [[100, 100],
             [600, 100],
             [500, 600]],
    colours: [Colour.defaultColour,
              Colour.defaultColour,
              Colour.defaultColour]
  },
  (self, renderer) => {
    return (params) => renderer.cmdRenderPoly(self.mergeWithDefaults(params));
  });


const Shapes = {
  publicBindings: [
    lineBinding,
    rectBinding,
    boxBinding,

    circleBinding,
    circleSliceBinding,
    splineBinding,

    bezierBinding,
    bezierTrailingBinding,
    bezierBulgingBinding,

    strokedBezierBinding,
    strokedBezierRectBinding,

    polyBinding
  ]
};

export default Shapes;
