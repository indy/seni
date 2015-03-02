/*jslint bitwise:true,maxparams:5,maxstatements:35*/

import PublicBinding from '../lang/PublicBinding';
import MathUtil from './MathUtil';
//import Colour from './Colour';
import Colour from './Colour';

let Format = Colour.Format;

function bezierPoint(a, b, c, d, t) {
  let t1 = 1 - t;

  return (a * t1 * t1 * t1) +
    (3 * b * t * t1 * t1) +
    (3 * c * t * t * t1) +
    (d * t * t * t);
}

function normals(x1, y1, x2, y2) {
  let dx = x2 - x1;
  let dy = y2 - y1;

  return [MathUtil.normalize(-dy, dx), MathUtil.normalize(dy, -dx)];
}

function _bezier(renderer) {
  const glContainer = renderer.getGLContainer();
  const buffer = renderer.getBuffer();

  return (params) => {

    let tessellation = params.tessellation || 15;
    let lineWidth = params.lineWidth || undefined;
    let lineWidthStart =
          params.lineWidthStart === undefined ? 20 : params.lineWidthStart;
    let lineWidthEnd =
          params.lineWidthEnd === undefined ? 20 : params.lineWidthEnd;
    let lineWidthMapping = params.lineWidthMapping || 'slow-in-out';
    let coords = params.coords || [[440, 400],
                                   [533, 700],
                                   [766, 200],
                                   [900, 500]];
    let tStart = params.tStart || 0;
    let tEnd = params.tEnd || 1;
    let colour = params.colour || undefined;

    let halfWidthEnd, remap;

    if(lineWidth !== undefined) {
      // user has given a constant lineWidth parameter
      halfWidthEnd  = lineWidth / 2.0;
      remap = () => halfWidthEnd;
    } else {
      // use the default start and end line widths
      let halfWidthStart  = lineWidthStart / 2.0;
      halfWidthEnd  = lineWidthEnd / 2.0;
      remap = MathUtil.remapFn({from: [tStart, tEnd],
                                to: [halfWidthStart, halfWidthEnd],
                                mapping: lineWidthMapping});

    }

    let tVals = MathUtil.stepsInclusive(tStart, tEnd, tessellation);
    let xs = tVals.map((i) => bezierPoint(coords[0][0],
                                          coords[1][0],
                                          coords[2][0],
                                          coords[3][0], i));
    let ys = tVals.map((i) => bezierPoint(coords[0][1],
                                          coords[1][1],
                                          coords[2][1],
                                          coords[3][1], i));

    if(colour === undefined) {
      colour = Colour.construct(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
    } else {
      colour = Colour.cloneAs(colour, Format.RGB);
    }

    let elementArray = Colour.elementArray(colour);

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
    (renderer) => {
      const glContainer = renderer.getGLContainer();
      const buffer = renderer.getBuffer();

      // return a function which accepts args as parameters
      return (params) => {

        let x = params.x || 0;
        let y = params.y || 0;
        let width = params.width || 100;
        let height = params.height || 100;
        let colour = params.colour || undefined;

        const halfWidth = (width / 2);
        const halfHeight = (height / 2);

        //console.log('rect: x:' + x + ', y:' + y + ', width:' + width + ', height:' + height);

        if(colour === undefined) {
          colour = Colour.construct(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
        } else {
          colour = Colour.cloneAs(colour, Format.RGB);
        }

        let elementArray = Colour.elementArray(colour);

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
    (renderer) => {
      return _bezier(renderer);
    }),

  bezierTrailing: new PublicBinding(
    'bezierTrailing',
    `
    this fn adds 1
    this is a multi line comment
    woo hoo
    `,
    (renderer) => {

      let bezierFn = _bezier(renderer);

      // return a function which accepts args as parameters
      return (params) => {

        let tessellation = params.tessellation || 15;
        let lineWidth = params.lineWidth || 20;
        let coords = params.coords || [[440, 400],
                                       [533, 700],
                                       [766, 200],
                                       [900, 500]];
        let tStart = params.tStart || 0;
        let tEnd = params.tEnd || 1;
        let colour = params.colour || undefined;

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
    (renderer) => {

      let bezierFn = _bezier(renderer);

      // return a function which accepts args as parameters
      return (params) => {


        let tessellation = params.tessellation || 16;
        let lineWidth = params.lineWidth || 20;
        let coords = params.coords || [[440, 400],
                                       [533, 700],
                                       [766, 200],
                                       [900, 500]];
        let tStart = params.tStart || 0;
        let tEnd = params.tEnd || 1;
        let colour = params.colour || undefined;

        let tMid = (tStart + tEnd) / 2.0;
        // tessellation should be an even number
        let newTess = tessellation & 1 ? tessellation + 1: tessellation;

        //let red = new Colour(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
        //renderRect(renderer, {x: coords[0][0], y: coords[0][1],
        //                      width: 20, height: 20, colour: red});
        //renderRect(renderer, {x: coords[3][0], y: coords[3][1],
        //                      width: 20, height: 20, colour: red});

        bezierFn({tessellation: newTess / 2,
                  lineWidthStart: 0.0,
                  lineWidthEnd: lineWidth,
                  coords: coords,
                  tStart: tStart,
                  tEnd: tMid,
                  colour: colour});
        bezierFn({tessellation: newTess / 2,
                  lineWidthStart: lineWidth,
                  lineWidthEnd: 0.0,
                  coords: coords,
                  tStart: tMid,
                  tEnd: tEnd,
                  colour: colour});
      };
    })
};

export default Shapes;
