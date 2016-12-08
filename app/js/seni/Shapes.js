/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
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

import PublicBinding from '../lang/PublicBinding';
import MathUtil from './MathUtil';
import Colour from './Colour';
import PseudoRandom from './PseudoRandom';

function renderSpline(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderQuadratic(fullParams);
}

function renderBezier(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderBezier(fullParams);
}

function renderBrushLine(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderBrushLine(fullParams);
}

function renderBrushStroke(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderBrushStroke(fullParams);
}

const bezierBinding = new PublicBinding(
  'bezier',
  { description: 'renders a bezier curve',
    args: [['tessellation', 'the number of polygons to use'],
           ['line-width', ''],
           ['line-width-start', ''],
           ['line-width-end', ''],
           ['line-width-mapping',
            'one of linear, quick, slow-in or slow-in-out'],
           ['coords', 'four control points'],
           ['t-start', '0'],
           ['t-end', '1'],
           ['colour', 'Colour.defaultColour']] },
  { tessellation: 15,
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
    colour: Colour.defaultColour },
  (self, renderer) => params => renderBezier(self, params, renderer));

function renderLine(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderLine(fullParams);
}

function renderRect(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderRect(fullParams);
}

function renderCircle(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderCircle(fullParams);
}

function renderBezierTrailing(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);

  const {tessellation,
         coords,
         colour} = fullParams;
  const lineWidth = fullParams['line-width'];
  const tStart = fullParams['t-start'];
  const tEnd = fullParams['t-end'];

  const bezierParams = {tessellation,
                        'line-width-start': lineWidth,
                        'line-width-end': 0.0,
                        'line-width-mapping': 'linear',
                        coords,
                        't-start': tStart,
                        't-end': tEnd,
                        colour};

  renderBezier(bezierBinding, bezierParams, renderer);
}

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

  const thinFatParams = {tessellation: newTess / 2,
                         'line-width-start': 0.0,
                         'line-width-end': lineWidth,
                         'line-width-mapping': 'slow-in-out',
                         coords,
                         't-start': tStart,
                         't-end': tMid,
                         colour};
  renderBezier(bezierBinding, thinFatParams, renderer);

  const fatThinParams = {tessellation: newTess / 2,
                         'line-width-start': lineWidth,
                         'line-width-end': 0.0,
                         'line-width-mapping': 'slow-in-out',
                         coords,
                         't-start': tMid,
                         't-end': tEnd,
                         colour};
  renderBezier(bezierBinding, fatThinParams, renderer);
}

const splineBinding = new PublicBinding(
  'spline',
  { description: 'renders a spline curve',
    args: [['tessellation', 'the number of polygons to use'],
           ['line-width', ''],
           ['line-width-start', ''],
           ['line-width-end', ''],
           ['line-width-mapping',
            'one of linear, quick, slow-in or slow-in-out'],
           ['coords', 'three control points'],
           ['t-start', '0'],
           ['t-end', '1'],
           ['colour', 'Colour.defaultColour']] },
  { tessellation: 15,
    'line-width': undefined,
    'line-width-start': 20,
    'line-width-end': 20,
    'line-width-mapping': 'slow-in-out',
    coords: [[440, 400],
             [533, 700],
             [766, 200]],
    't-start': 0,
    't-end': 1,
    colour: Colour.defaultColour },
  (self, renderer) => params => renderSpline(self, params, renderer));

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

  const [[x1, y1], [x2, y2], [x3, y3], [x4, y4]] = coords;
  const tv = MathUtil.stepsInclusive(0.0, 1.0, tessellation + 2);

  const lab = Colour.cloneAs(colour, Colour.Format.LAB);

  /* eslint-disable no-loop-func */
  for (let i = 0; i < tessellation; i++) {
    const tvals = [tv[i + 0], tv[i + 1], tv[i + 2]];
    // get 3 points on the bezier curve
    const [xx1, xx2, xx3] =
          tvals.map(t => MathUtil.bezierPoint(x1, x2, x3, x4, t));
    const [yy1, yy2, yy3] =
          tvals.map(t => MathUtil.bezierPoint(y1, y2, y3, y4, t));

    const ns = strokeNoise;

    const colLabL = Colour.getComponent(lab, Colour.L) +
          (PseudoRandom._perlin(xx1, xx1, xx1) * colourVolatility);

    const splineParams = {
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
  { description: 'renders a stroked bezier curve',
    args: [['tessellation', 'the number of polygons to use'],
           ['line-width', ''],
           ['coords', 'four control points'],
           ['stroke-tessellation', 'the tessellation of each basic bezier'],
           ['stroke-noise',
            'the amount of noise to add to each basic bezier curve'],
           ['stroke-line-width-start', 'the width of each basic bezier'],
           ['stroke-line-width-end', 'the width of each basic bezier'],
           ['colour', 'Colour.defaultColour'],
           ['colour-volatility', '1'],
           ['seed', 'the random seed']] },
  { tessellation: 15,
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

    seed: 42 },
  (self, renderer) => params => renderStrokedBezier(self, params, renderer));

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
  const strokeTessellation = fullParams['stroke-tessellation'];
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
    const hParams = {
      tessellation,
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
    const vParams = {
      tessellation,
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

function renderCircleSlice(publicBinding, params, renderer) {
  const fullParams = publicBinding.mergeWithDefaults(params);
  renderer.cmdRenderCircleSlice(fullParams);
}

const rectBinding = new PublicBinding(
  'rect',
  { description:
    'renders a rectangle, centered in position with the given width, height',
    args: [['position', 'a position vector'],
           ['width', 'width'],
           ['height', 'height'],
           ['colour', 'Colour.defaultColour']] },
  { position: [500, 500],
    width: 200,
    height: 200,
    colour: Colour.defaultColour },
  (self, renderer) => params => renderRect(self, params, renderer));

const publicBindings = [
  rectBinding,
  splineBinding,
  bezierBinding,

  new PublicBinding(
    'bezier-trailing',
    { description: 'renders a trailing bezier curve',
      args: [['tessellation', 'the number of polygons to use'],
             ['line-width', ''],
             ['coords', 'three control points'],
             ['t-start', '0'],
             ['t-end', '1'],
             ['colour', 'Colour.defaultColour']] },
    { tessellation: 15,
      'line-width': 20,
      coords: [[440, 400],
               [533, 700],
               [766, 200],
               [900, 500]],
      't-start': 0,
      't-end': 1,
      colour: Colour.defaultColour },
    (self, renderer) => params => renderBezierTrailing(self, params, renderer)),

  new PublicBinding(
    'bezier-bulging',
    { description: 'renders a bulging bezier curve',
      args: [['tessellation', 'the number of polygons to use'],
             ['line-width', ''],
             ['coords', 'three control points'],
             ['t-start', '0'],
             ['t-end', '1'],
             ['colour', 'Colour.defaultColour']] },
    { tessellation: 16,
      'line-width': 20,
      coords: [[440, 400],
               [533, 700],
               [766, 200],
               [900, 500]],
      't-start': 0,
      't-end': 1,
      colour: Colour.defaultColour },
    (self, renderer) => params => renderBezierBulging(self, params, renderer)),

  new PublicBinding(
    'brush-line',
    { description: 'renders a brush line',
      args: [['from', 'vector'],
             ['to', 'vector'],
             ['width', 'the width of the brush'],
             ['colour', 'Colour.defaultColour'],
             ['brush-type', 'the type of texture to use'],
             ['brush-subtype', 'the subtype of texture to use']] },
    { from: [100, 100],
      to: [500, 500],
      width: 20,
      colour: Colour.defaultColour,
      'brush-type': 'brushA',
      'brush-subtype': 0},
    (self, renderer) => params => renderBrushLine(self, params, renderer)),

  new PublicBinding(
    'brush-stroke',
    { description: 'renders a brush stroke alone a bezier curve',
      args: [['tessellation', 'the number of polygons to use'],
             ['line-width', ''],
             ['line-width-start', ''],
             ['line-width-end', ''],
             ['line-width-mapping',
              'one of linear, quick, slow-in or slow-in-out'],
             ['coords', 'four control points'],
             ['t-start', '0'],
             ['t-end', '1'],
             ['colour', 'Colour.defaultColour'],
             ['brush-type', 'the type of texture to use'],
             ['brush-subtype', 'the subtype of texture to use']] },
    { tessellation: 15,
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
      colour: Colour.defaultColour,
      'brush-type': 'brushA',
      'brush-subtype': 0},
    (self, renderer) => params => renderBrushStroke(self, params, renderer)),

  strokedBezierBinding,

  new PublicBinding(
    'line',
    { description: "renders a line using 'from' and 'to' parameters",
      args: [['from', 'vector'],
             ['to', 'vector'],
             ['width', '20'],
             ['colour', 'Colour.defaultColour']] },
    { from: [100, 100],
      to: [500, 500],
      width: 20,
      colour: Colour.defaultColour },
    (self, renderer) => params => renderLine(self, params, renderer)),

  new PublicBinding(
    'box',
    { description:
      'renders a rectangle using the given top, left, bottom, right parameters',
      args: [['top', ''],
             ['left', ''],
             ['bottom', ''],
             ['right', ''],
             ['colour', 'Colour.defaultColour']] },
    { top: 300,
      left: 100,
      bottom: 100,
      right: 300,
      colour: Colour.defaultColour },
    (self, renderer) => params => {
      const fullParams = self.mergeWithDefaults(params);
      const width = fullParams.right - fullParams.left;
      const height = fullParams.top - fullParams.bottom;
      const rectParams = {
        position: [fullParams.left + (width / 2),
                   fullParams.bottom + (height / 2)],
        width,
        height,
        colour: fullParams.colour
      };
      renderRect(rectBinding, rectParams, renderer);
    }),

  new PublicBinding(
    'circle',
    { description: 'renders a circle',
      args: [['position', ''],
             ['radius', ''],
             ['width', ''],
             ['height', ''],
             ['tessellation', ''],
             ['colour', 'Colour.defaultColour']] },
    { position: [500, 500],
      radius: undefined,
      width: 200,
      height: 200,
      tessellation: 10,
      colour: Colour.defaultColour },
    (self, renderer) => params => renderCircle(self, params, renderer)),

  new PublicBinding(
    'circle-slice',
    { description: 'renders a circle-slice',
      args: [['position', ''],
             ['radius', ''],
             ['angle-start', ''],
             ['angle-end', ''],
             ['width', ''],
             ['height', ''],
             ['inner-width', ''],
             ['inner-height', ''],
             ['tessellation', ''],
             ['colour', 'Colour.defaultColour']] },
    { position: [500, 500],
      radius: undefined,
      'angle-start': 0,
      'angle-end': 180,
      width: 200,
      height: 200,
      'inner-width': 0,
      'inner-height': 0,
      tessellation: 10,
      colour: Colour.defaultColour },
    (self, renderer) => params => renderCircleSlice(self, params, renderer)),

  new PublicBinding(
    'stroked-bezier-rect',
    { description: 'renders a stroked Bezier rect',
      args: [['position', ''],
             ['colour', 'Colour.defaultColour']] },
    { position: [100, 100],
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
      'colour-volatility': 40 },
    (self, renderer) => params =>
      renderStrokedBezierRect(self, params, renderer)),

  new PublicBinding(
    'poly',
    { description:
      'renders triangle strip in which each vertex has a different colour',
      args: [['coords', '3 points'],
             ['colours', '3 colours']] },
    { coords: [[100, 100],
               [600, 100],
               [500, 600]],
      colours: [Colour.defaultColour,
                Colour.defaultColour,
                Colour.defaultColour] },
    (self, renderer) => params =>
      renderer.cmdRenderPoly(self.mergeWithDefaults(params)))];

export default {
  publicBindingType: 'binding',
  publicBindings
};
