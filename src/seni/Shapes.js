/*jslint ignore:start*/

/*jslint bitwise:true,maxparams:5,maxstatements:35*/

import PublicBinding from '../lang/PublicBinding';
import MathUtil from './MathUtil';
import Colour from './Colour';
import Util from './Util';
import Perlin from './Perlin';

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

const bezierDefaultParams = {tessellation: 15,
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
                             colour: Colour.defaultColour};


function _bezier(renderer) {
  const glContainer = renderer.getGLContainer();
  const buffer = renderer.getBuffer();

  return (params) => {

    const {
      tessellation,
      lineWidth,
      lineWidthStart,
      lineWidthEnd,
      lineWidthMapping,
      coords,
      tStart,
      tEnd
    } = params;

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

  };
}

function _bezierBezier(renderer) {

  let bezierFn = _bezier(renderer);

  return (params) => {

    const {
      tessellation,
      lineWidth,
      coords,
      strokeWidth,
      strokeTessellation,
      strokeNoise,
      colour
    } = params;

    let [[x1, y1], [x2, y2], [x3, y3], [x4, y4]] = coords;
    let tv = MathUtil.stepsInclusive(0.0, 1.0, tessellation + 3);
    for(let i=0;i<tessellation;i++) {
      for(let w=3;w < (lineWidth/2); w += 12) { // ????

        let tvals = [tv[i+0], tv[i+1], tv[i+2], tv[i+3]];
        let [xx1, xx2, xx3, xx4] =
              tvals.map((t) => bezierPoint(x1, x2, x3, x4, t));
        let [yy1, yy2, yy3, yy4] =
              tvals.map((t) => bezierPoint(y1, y2, y3, y4, t));
        let [[xn1, yn1], [xn2, yn2]] = normals(xx1, yy1, xx4, yy4);
        let ns = strokeNoise;

        bezierFn(Util.merge({
          tessellation: strokeTessellation,
          lineWidth: strokeWidth,
          colour: colour,
          coords: [
            [(xx1 + (xn1 * w) + (ns * Perlin._perlin(xx1, xn1, w))),
             (yy1 + (yn1 * w) + (ns * Perlin._perlin(yy1, yn1, w)))],

            [(xx2 + (xn1 * w) + (ns * Perlin._perlin(xx2, xn1, w))),
             (yy2 + (yn1 * w) + (ns * Perlin._perlin(yy2, yn1, w)))],

            [(xx3 + (xn1 * w) + (ns * Perlin._perlin(xx3, xn1, w))),
             (yy3 + (yn1 * w) + (ns * Perlin._perlin(yy3, yn1, w)))],

            [(xx4 + (xn1 * w) + (ns * Perlin._perlin(xx4, xn1, w))),
             (yy4 + (yn1 * w) + (ns * Perlin._perlin(yy4, yn1, w)))],
          ]
        }, bezierDefaultParams));

        bezierFn(Util.merge({
          tessellation: strokeTessellation,
          lineWidth: strokeWidth,
          colour: colour,
          coords: [
            [(xx1 + (xn2 * w) + (ns * Perlin._perlin(xx1, xn2, w))),
             (yy1 + (yn2 * w) + (ns * Perlin._perlin(yy1, yn2, w)))],

            [(xx2 + (xn2 * w) + (ns * Perlin._perlin(xx2, xn2, w))),
             (yy2 + (yn2 * w) + (ns * Perlin._perlin(yy2, yn2, w)))],

            [(xx3 + (xn2 * w) + (ns * Perlin._perlin(xx3, xn2, w))),
             (yy3 + (yn2 * w) + (ns * Perlin._perlin(yy3, yn2, w)))],

            [(xx4 + (xn2 * w) + (ns * Perlin._perlin(xx4, xn2, w))),
             (yy4 + (yn2 * w) + (ns * Perlin._perlin(yy4, yn2, w)))],
          ]
        }, bezierDefaultParams));
      }
    }

  };
}

const Shapes = {
  rect: new PublicBinding(
    'rect',
    `
    this fn adds 1
    this is a multi line comment
    woo hoo
    `,

    {x: 0.0,
     y: 0.0,
     width: 100,
     height: 100,
     colour: Colour.defaultColour},

    (self, renderer) => {
      const glContainer = renderer.getGLContainer();
      const buffer = renderer.getBuffer();

      // return a function which accepts args as parameters
      return (params) => {
        let {x, y, width, height, colour} = self.mergeWithDefaults(params);

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
      };
    }),

  bezier: new PublicBinding(
    'bezier',
    `
    this fn adds 1
    this is a multi line comment
    woo hoo
    `,

    bezierDefaultParams,

    (self, renderer) => {
      const bezierFn = _bezier(renderer);
      return (params) => {
        bezierFn(self.mergeWithDefaults(params));
      };
    }),

  bezierTrailing: new PublicBinding(
    'bezierTrailing',
    `
    this fn adds 1
    this is a multi line comment
    woo hoo
    `,

    {tessellation: 15,
     lineWidth: 20,
     coords: [[440, 400],
              [533, 700],
              [766, 200],
              [900, 500]],
     tStart: 0,
     tEnd: 1,
     colour: Colour.defaultColour},

    (self, renderer) => {

      const bezierFn = _bezier(renderer);

      // return a function which accepts args as parameters
      return (params) => {

        const {tessellation,
               lineWidth,
               coords,
               tStart,
               tEnd,
               colour} = self.mergeWithDefaults(params);

        bezierFn({tessellation: tessellation,
                  lineWidthStart: lineWidth,
                  lineWidthEnd: 0.0,
                  lineWidthMapping: 'linear',
                  coords: coords,
                  tStart: tStart,
                  tEnd: tEnd,
                  colour: colour});
      };
    }),

  bezierBulging: new PublicBinding(
    'bezierBulging',
    `
    this fn adds 1
    this is a multi line comment
    woo hoo
    `,

    {tessellation: 16,
     lineWidth: 20,
     coords: [[440, 400],
              [533, 700],
              [766, 200],
              [900, 500]],
     tStart: 0,
     tEnd: 1,
     colour: Colour.defaultColour},

    (self, renderer) => {

      const bezierFn = _bezier(renderer);

      // return a function which accepts args as parameters
      return (params) => {

        const {tessellation,
               lineWidth,
               coords,
               tStart,
               tEnd,
               colour} = self.mergeWithDefaults(params);

        const tMid = (tStart + tEnd) / 2.0;
        // tessellation should be an even number
        const newTess = tessellation & 1 ? tessellation + 1: tessellation;

        bezierFn({tessellation: newTess / 2,
                  lineWidthStart: 0.0,
                  lineWidthEnd: lineWidth,
                  lineWidthMapping: 'slow-in-out',
                  coords: coords,
                  tStart: tStart,
                  tEnd: tMid,
                  colour: colour});
        bezierFn({tessellation: newTess / 2,
                  lineWidthStart: lineWidth,
                  lineWidthEnd: 0.0,
                  lineWidthMapping: 'slow-in-out',
                  coords: coords,
                  tStart: tMid,
                  tEnd: tEnd,
                  colour: colour});
      };
    }),

    bezierBezier: new PublicBinding(
    'bezierbezier',
    `
    `,

    {tessellation: 15,
     lineWidth: 20,

     coords: [[440, 400],
              [533, 700],
              [766, 200],
              [900, 500]],

     strokeWidth: 30,
     strokeTessellation: 10,
     strokeNoise: 25,

     colour: Colour.defaultColour},

    (self, renderer) => {
      const bezierFn = _bezierBezier(renderer);
      return (params) => {
        bezierFn(self.mergeWithDefaults(params));
      };
    })
};

export default Shapes;
