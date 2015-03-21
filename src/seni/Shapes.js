/*jslint bitwise:true,maxparams:5,maxstatements:35*/

import PublicBinding from '../lang/PublicBinding';
import MathUtil from './MathUtil';
import Colour from './Colour';

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

    {tessellation: 15,
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
     colour: Colour.defaultColour},

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
    })
};

export default Shapes;
