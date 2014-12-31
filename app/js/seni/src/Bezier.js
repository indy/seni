import { GLContainer } from './GLContainer';
import { Buffer } from './Buffer';
import {
  normalize,
  stepsInclusive
} from './MathUtil';

export class Bezier {
  doubler(d) {
    let answer = d * 2;
    return answer;
  }

  foofoo(a, b, ...rest) {
    return a + b;
  }
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

export function getBezierFn(glContainer, buffer) {
  return function(params) {
    renderBezier(glContainer, buffer, params);
  }
}

export function renderBezier(glContainer,
                             buffer,
                             {tessellation = 15,
                              lineWidth = 20,
                              coords = [[440, 400],
                                        [533, 700],
                                        [766, 200],
                                        [900, 500]],
                              tStart = 0,
                              tEnd = 1}) {

  let halfWidth = lineWidth / 2;
  let tVals = stepsInclusive(tStart, tEnd, tessellation);

  let xs = tVals.map((i) => bezierPoint(coords[0][0],
                                        coords[1][0],
                                        coords[2][0],
                                        coords[3][0], i));
  let ys = tVals.map((i) => bezierPoint(coords[0][1],
                                        coords[1][1],
                                        coords[2][1],
                                        coords[3][1], i));

  let c = [1.0, 0.0, 0.0, 1.0]; // colour

  for(let i=0; i<tVals.length - 1; i++) {  
    let [[xn1, yn1], [xn2, yn2]] = normals(xs[i], ys[i], xs[i+1], ys[i+1]),
        i0 = xs[i],
        i1 = ys[i];

    if(i === 0) {
      buffer.prepareToAddTriangleStrip(glContainer, tessellation * 2,
                                       [(xn1 * halfWidth) + i0,
                                        (yn1 * halfWidth) + i1]);
    }

    buffer.addVertex([(xn1 * halfWidth) + i0, (yn1 * halfWidth) + i1], c);
    buffer.addVertex([(xn2 * halfWidth) + i0, (yn2 * halfWidth) + i1], c);
  }

  // final 2 vertices for the end point
  let i = tVals.length - 2,
      [[xn1, yn1], [xn2, yn2]] = normals(xs[i], ys[i], xs[i+1], ys[i+1]),
      i2 = xs[i+1],
      i3 = ys[i+1];
  
  buffer.addVertex([(xn1 * halfWidth) + i2, (yn1 * halfWidth) + i3], c);
  buffer.addVertex([(xn2 * halfWidth) + i2, (yn2 * halfWidth) + i3], c);
}

