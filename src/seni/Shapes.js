/*jslint bitwise:true,maxparams:6,maxstatements:50*/

import PublicBinding from '../lang/PublicBinding';
import MathUtil from './MathUtil';
import Colour from './Colour';
import Perlin from './Perlin';
import SeedRandom from './SeedRandom';

const Format = Colour.Format;

function bezierPoint(a, b, c, d, t) {
  const t1 = 1 - t;

  return (a * t1 * t1 * t1) +
    (3 * b * t * t1 * t1) +
    (3 * c * t * t * t1) +
    (d * t * t * t);
}

function normals(x1, y1, x2, y2) {
  const dx = x2 - x1;
  const dy = y2 - y1;

  return [MathUtil.normalize(-dy, dx), MathUtil.normalize(dy, -dx)];
}

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

function renderSpline(publicBinding, renderer, params) {
  const glContainer = renderer.getGLContainer();
  const buffer = renderer.getBuffer();

  const {
    colour,
    coords,
    tessellation,
    tStart,
    tEnd,
    lineWidth,
    lineWidthStart,
    lineWidthEnd,
    lineWidthMapping
  } = publicBinding.mergeWithDefaults(params);

  let halfWidthEnd, remap;

  if(lineWidth !== undefined) {
    // user has given a constant lineWidth parameter
    halfWidthEnd  = lineWidth / 2.0;
    remap = () => halfWidthEnd;
  } else {
    // use the default start and end line widths
    const halfWidthStart  = lineWidthStart / 2.0;
    halfWidthEnd  = lineWidthEnd / 2.0;
    remap = MathUtil.remapFn({from: [tStart, tEnd],
                              to: [halfWidthStart, halfWidthEnd],
                              mapping: lineWidthMapping});

  }


  const elementArray = Colour.elementArray(colour);

  let debugRendering = false;
  if(debugRendering) {
    const c = coords;
    for(let i=0;i<3;i++) {
      debugRect(buffer, glContainer, c[i][0], c[i][1], 10, colour);
    }
  }

  const p = coords;
  const a0x = p[0][0];
  const a0y = p[0][1];
  const a2x = ((p[1][0] - p[0][0]) - 0.5*(p[2][0] - p[0][0])) / (0.5 * (0.5-1));
  const a2y = ((p[1][1] - p[0][1]) - 0.5*(p[2][1] - p[0][1])) / (0.5 * (0.5-1));
  const a1x = p[2][0] - p[0][0] - a2x;
  const a1y = p[2][1] - p[0][1] - a2y;

  const tVals = MathUtil.stepsInclusive(tStart, tEnd, tessellation);
  const xs = tVals.map((t) => (a2x*t*t) + (a1x*t) + a0x);
  const ys = tVals.map((t) => (a2y*t*t) + (a1y*t) + a0y);

  for(let i=0; i<tVals.length - 1; i++) {
    let [[xn1, yn1], [xn2, yn2]] = normals(xs[i], ys[i], xs[i+1], ys[i+1]),
        i0 = xs[i],
        i1 = ys[i],
        t = tVals[i];

    if(i === 0) {
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
      [[xn1, yn1], [xn2, yn2]] = normals(xs[i], ys[i], xs[i+1], ys[i+1]),
      i2 = xs[i+1],
      i3 = ys[i+1];

  buffer.addVertex([(xn1 * halfWidthEnd) + i2,
                    (yn1 * halfWidthEnd) + i3,
                    0.0],
                   elementArray);
  buffer.addVertex([(xn2 * halfWidthEnd) + i2,
                    (yn2 * halfWidthEnd) + i3,
                    0.0],
                   elementArray);


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
  const glContainer = renderer.getGLContainer();
  const buffer = renderer.getBuffer();

  const {
    tessellation,
    lineWidth,
    lineWidthStart,
    lineWidthEnd,
    lineWidthMapping,
    coords,
    tStart,
    tEnd
  } = publicBinding.mergeWithDefaults(params);

  let {colour} = params;

  let halfWidthEnd, remap;

  if(lineWidth !== undefined) {
    // user has given a constant lineWidth parameter
    halfWidthEnd  = lineWidth / 2.0;
    remap = () => halfWidthEnd;
  } else {
    // use the default start and end line widths
    const halfWidthStart  = lineWidthStart / 2.0;
    halfWidthEnd  = lineWidthEnd / 2.0;
    remap = MathUtil.remapFn({from: [tStart, tEnd],
                              to: [halfWidthStart, halfWidthEnd],
                              mapping: lineWidthMapping});

  }

  const tVals = MathUtil.stepsInclusive(tStart, tEnd, tessellation);
  const xs = tVals.map((i) => bezierPoint(coords[0][0],
                                          coords[1][0],
                                          coords[2][0],
                                          coords[3][0], i));
  const ys = tVals.map((i) => bezierPoint(coords[0][1],
                                          coords[1][1],
                                          coords[2][1],
                                          coords[3][1], i));

  if(colour === undefined) {
    colour = Colour.construct(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
  } else {
    colour = Colour.cloneAs(colour, Format.RGB);
  }

  const elementArray = Colour.elementArray(colour);

  for(let i=0; i<tVals.length - 1; i++) {
    let [[xn1, yn1], [xn2, yn2]] = normals(xs[i], ys[i], xs[i+1], ys[i+1]),
        i0 = xs[i],
        i1 = ys[i],
        t = tVals[i];

    if(i === 0) {
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
      [[xn1, yn1], [xn2, yn2]] = normals(xs[i], ys[i], xs[i+1], ys[i+1]),
      i2 = xs[i+1],
      i3 = ys[i+1];

  buffer.addVertex([(xn1 * halfWidthEnd) + i2,
                    (yn1 * halfWidthEnd) + i3,
                    0.0],
                   elementArray);
  buffer.addVertex([(xn2 * halfWidthEnd) + i2,
                    (yn2 * halfWidthEnd) + i3,
                    0.0],
                   elementArray);
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

  //console.log('rect: x:' + x + ', y:' + y + ', width:' + width + ', height:' + height);

  if(colour === undefined) {
    colour = Colour.construct(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
  } else {
    colour = Colour.cloneAs(colour, Format.RGB);
  }

  const elementArray = Colour.elementArray(colour);

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
  this is a multi line comment
  woo hoo
  `,
  {
    x: 0.0,
    y: 0.0,
    width: 100,
    height: 100,
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
  const newTess = tessellation & 1 ? tessellation + 1: tessellation;

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


function renderBezierScratch(publicBinding, renderer, params) {
  const {
    tessellation,
    lineWidth,
    coords,
    strokeWidth,
    strokeTessellation,
    strokeNoise,
    colour,
    colourVolatility,
    seed
  } = publicBinding.mergeWithDefaults(params);

  let [[x1, y1], [x2, y2], [x3, y3], [x4, y4]] = coords;
  let tv = MathUtil.stepsInclusive(0.0, 1.0, tessellation + 3);

  let lab = Colour.cloneAs(colour, Colour.Format.LAB);

  for(let i=0;i<tessellation;i++) {
    for(let w=3;w < (lineWidth/2); w += 12) { // ????

      let tvals = [tv[i+0], tv[i+1], tv[i+2], tv[i+3]];
      let [xx1, xx2, xx3, xx4] =
            tvals.map((t) => bezierPoint(x1, x2, x3, x4, t));
      let [yy1, yy2, yy3, yy4] =
            tvals.map((t) => bezierPoint(y1, y2, y3, y4, t));
      let [[xn1, yn1], [xn2, yn2]] = normals(xx1, yy1, xx4, yy4);
      let ns = strokeNoise;

      renderBezier(bezierBinding, renderer, {
        tessellation: strokeTessellation,
        lineWidth: strokeWidth,
        colour: Colour.setLightness(lab, Colour.getLightness(lab) +
                                    (Perlin._perlin(xx1, xn1, w) * colourVolatility)),
        coords: [
          [(xx1 + (xn1 * w) + (ns * Perlin._perlin(xx1, xn1, w * seed))),
           (yy1 + (yn1 * w) + (ns * Perlin._perlin(yy1, yn1, w * seed)))],

          [(xx2 + (xn1 * w) + (ns * Perlin._perlin(xx2, xn1, w * seed))),
           (yy2 + (yn1 * w) + (ns * Perlin._perlin(yy2, yn1, w * seed)))],

          [(xx3 + (xn1 * w) + (ns * Perlin._perlin(xx3, xn1, w * seed))),
           (yy3 + (yn1 * w) + (ns * Perlin._perlin(yy3, yn1, w * seed)))],

          [(xx4 + (xn1 * w) + (ns * Perlin._perlin(xx4, xn1, w * seed))),
           (yy4 + (yn1 * w) + (ns * Perlin._perlin(yy4, yn1, w * seed)))],
        ]
      });

      renderBezier(bezierBinding, renderer, {
        tessellation: strokeTessellation,
        lineWidth: strokeWidth,
        colour: Colour.setLightness(lab, Colour.getLightness(lab) +
                                    (Perlin._perlin(xx1, xn2, w) * colourVolatility)),
        coords: [
          [(xx1 + (xn2 * w) + (ns * Perlin._perlin(xx1, xn2, w * seed))),
           (yy1 + (yn2 * w) + (ns * Perlin._perlin(yy1, yn2, w * seed)))],

          [(xx2 + (xn2 * w) + (ns * Perlin._perlin(xx2, xn2, w * seed))),
           (yy2 + (yn2 * w) + (ns * Perlin._perlin(yy2, yn2, w * seed)))],

          [(xx3 + (xn2 * w) + (ns * Perlin._perlin(xx3, xn2, w * seed))),
           (yy3 + (yn2 * w) + (ns * Perlin._perlin(yy3, yn2, w * seed)))],

          [(xx4 + (xn2 * w) + (ns * Perlin._perlin(xx4, xn2, w * seed))),
           (yy4 + (yn2 * w) + (ns * Perlin._perlin(yy4, yn2, w * seed)))],
        ]
      });
    }
  }
}

const bezierScratchBinding = new PublicBinding(
  'bezierScratch',
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
    lineWidth: 20,

    coords: [[440, 400],
             [533, 700],
             [766, 200],
             [900, 500]],

    strokeWidth: 30,
    strokeTessellation: 10,
    strokeNoise: 25,

    colour: Colour.defaultColour,
    colourVolatility: 0,

    seed: 42
  },
  (self, renderer) => {
    return (params) => renderBezierScratch(self, renderer, params);
  });

function renderBezierScratchRect(publicBinding, renderer, params) {
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
    strokeWidth,
    strokeTessellation,
    strokeNoise,
    colour,
    colourVolatility
  } = publicBinding.mergeWithDefaults(params);

  let thWidth = width / 3;
  let thHeight = height / 3;
  let vol = volatility;

  let hDelta = height / iterations;
  let hStripWidth = height / iterations;
  let halfHStripWidth = hStripWidth / 2;

  let vDelta = width / iterations;
  let vStripWidth = width / iterations;
  let halfVStripWidth = vStripWidth / 2;

  let halfAlphaCol = Colour.cloneAs(colour, Colour.Format.LAB);
  let lab = Colour.setAlpha(halfAlphaCol, Colour.getAlpha(halfAlphaCol) / 2);

  let rng = SeedRandom.buildSigned(seed);
  let i;

  for(i=iterations; i>0; i--) {
    renderBezierScratch(bezierScratchBinding, renderer, {
      tessellation: tessellation,
      lineWidth: overlap + hStripWidth,
      coords: [
        [(rng() * vol) + x + (0 * thWidth),
         ((i * hDelta) + (rng() * vol) + y) - halfHStripWidth],
        [(rng() * vol) + x + (1 * thWidth),
         ((i * hDelta) + (rng() * vol) + y) - halfHStripWidth],
        [(rng() * vol) + x + (2 * thWidth),
         ((i * hDelta) + (rng() * vol) + y) - halfHStripWidth],
        [(rng() * vol) + x + (3 * thWidth),
         ((i * hDelta) + (rng() * vol) + y) - halfHStripWidth]
      ],
      strokeWidth: strokeWidth,
      strokeTessellation: strokeTessellation,
      strokeNoise: strokeNoise,
      colour: lab,
      colourVolatility: colourVolatility
    });
  }

  for(i=iterations; i>0; i--) {
    renderBezierScratch(bezierScratchBinding, renderer, {
      tessellation: tessellation,
      lineWidth: overlap + vStripWidth,
      coords: [
        [((i * vDelta) + (rng() * vol) + x) - halfVStripWidth,
         (rng() * vol) + y + (0 * thHeight)],
        [((i * vDelta) + (rng() * vol) + x) - halfVStripWidth,
         (rng() * vol) + y + (1 * thHeight)],
        [((i * vDelta) + (rng() * vol) + x) - halfVStripWidth,
         (rng() * vol) + y + (2 * thHeight)],
        [((i * vDelta) + (rng() * vol) + x) - halfVStripWidth,
         (rng() * vol) + y + (3 * thHeight)]
      ],
      strokeWidth: strokeWidth,
      strokeTessellation: strokeTessellation,
      strokeNoise: strokeNoise,
      colour: lab,
      colourVolatility: colourVolatility
    });
  }
}

const bezierScratchRectBinding = new PublicBinding(
  'bezierScratchRect',
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

    strokeWidth: 30,
    strokeTessellation: 10,
    strokeNoise: 25,

    colour: Colour.defaultColour,
    colourVolatility: 40
  },
  (self, renderer) => {
    return (params) => renderBezierScratchRect(self, renderer, params);
  });

const Shapes = {
  rect: rectBinding,
  spline: splineBinding,
  bezier: bezierBinding,
  bezierTrailing: bezierTrailingBinding,
  bezierBulging: bezierBulgingBinding,
  bezierScratch: bezierScratchBinding,
  bezierScratchRect: bezierScratchRectBinding
};

export default Shapes;
