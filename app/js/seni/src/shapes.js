import { GLContainer } from './GLContainer';
import { Buffer } from './Buffer';
import {
  normalize,
  stepsInclusive,
  remapFn
} from './MathUtil';
import * as Colour from './Colour';


// partitions the given arguments into an array containing pairs of co-ordinates
export function coords(...args) {

  let res = [];
  for(let i=0;i<args.length;i+=2) {
    res.push([args[i], args[i+1]]);
  }
  console.log(res);
  //return res;

  console.log("hello from coords");
  return res;
}

export function renderBezier(renderer,
                             {tessellation = 15,
                              lineWidth = undefined,
                              lineWidthStart = 20,
                              lineWidthEnd = 20,
                              lineWidthMapping = "slow-in-out",
                              coords = [[440, 400],
                                        [533, 700],
                                        [766, 200],
                                        [900, 500]],
                              tStart = 0,
                              tEnd = 1,
                              colour = undefined}) {

  var glContainer = renderer.getGLContainer();
  var buffer = renderer.getBuffer();

  var halfWidthEnd, remap;

  if(lineWidth !== undefined) {
    // user has given a constant lineWidth parameter
    halfWidthEnd  = lineWidth / 2.0;
    remap = () => halfWidthEnd;

  } else {
    // use the default start and end line widths
    let halfWidthStart  = lineWidthStart / 2.0;
    halfWidthEnd  = lineWidthEnd / 2.0;
    remap = remapFn({from: [tStart, tEnd],
                     to: [halfWidthStart, halfWidthEnd],
                     mapping: lineWidthMapping});
    
  }

  let tVals = stepsInclusive(tStart, tEnd, tessellation);

  let xs = tVals.map((i) => bezierPoint(coords[0][0],
                                        coords[1][0],
                                        coords[2][0],
                                        coords[3][0], i));
  let ys = tVals.map((i) => bezierPoint(coords[0][1],
                                        coords[1][1],
                                        coords[2][1],
                                        coords[3][1], i));

  if(colour === undefined) {
    colour = {
      format: Colour.Format.RGB,
      val:  [1.0, 0.0, 0.0, 1.0]
    };
  } else if(colour.format !== Colour.Format.RGB) {
    colour = Colour.cloneAs(colour, Colour.Format.RGB).val;
  }
  
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

    buffer.addVertex([(xn1 * remap({val: t})) + i0, (yn1 * remap({val: t})) + i1, 0.0], 
                     colour.val);
    buffer.addVertex([(xn2 * remap({val: t})) + i0, (yn2 * remap({val: t})) + i1, 0.0], 
                     colour.val);
  }

  // final 2 vertices for the end point
  let i = tVals.length - 2,
      [[xn1, yn1], [xn2, yn2]] = normals(xs[i], ys[i], xs[i+1], ys[i+1]),
      i2 = xs[i+1],
      i3 = ys[i+1];
  
  buffer.addVertex([(xn1 * halfWidthEnd) + i2, (yn1 * halfWidthEnd) + i3, 0.0], 
                   colour.val);
  buffer.addVertex([(xn2 * halfWidthEnd) + i2, (yn2 * halfWidthEnd) + i3, 0.0],
                   colour.val);
}

export function renderBezierTrailing(renderer,
                                     {tessellation = 15,
                                      lineWidth = 20,
                                      coords = [[440, 400],
                                                [533, 700],
                                                [766, 200],
                                                [900, 500]],
                                      tStart = 0,
                                      tEnd = 1,
                                      colour = undefined}) {
  renderBezier(renderer, {tessellation: tessellation,
                          lineWidthStart: lineWidth,
                          lineWidthEnd: 0.0,
                          lineWidthMapping: "linear",
                          coords: coords,
                          tStart: tStart,
                          tEnd: tEnd});
}

export function renderBezierBulging(renderer,
                                     {tessellation = 15,
                                      lineWidth = 20,
                                      coords = [[440, 400],
                                                [533, 700],
                                                [766, 200],
                                                [900, 500]],
                                      tStart = 0,
                                      tEnd = 1,
                                      colour = undefined}) {

  let tMid = (tStart + tEnd) / 2.0;
  
  renderBezier(renderer, {tessellation: tessellation / 2.0,
                          lineWidthStart: 0.0,
                          lineWidthEnd: lineWidth,
                          coords: coords,
                          tStart: tStart,
                          tEnd: tMid});
  renderBezier(renderer, {tessellation: tessellation / 2.0,
                          lineWidthStart: lineWidth,
                          lineWidthEnd: 0.0,
                          coords: coords,
                          tStart: tMid,
                          tEnd: tEnd});
}


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

  return [normalize(-dy, dx), normalize(dy, -dx)];
}
