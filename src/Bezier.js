import { GLContainer } from './GLContainer';
import { Buffer } from './Buffer';

export class Bezier {
  doubler(d) {
    return d * 2;
  }

  foofoo(a, b, ...rest) {
    return a + b;
  }
}


function bezierPoint(a, b, c, d, t) {
    var t1 = 1 - t;

    return (a * t1 * t1 * t1) +
        (3 * b * t * t1 * t1) +
        (3 * c * t * t * t1) +
        (d * t * t * t);
}


function normals(x1, y1, x2, y2) {
    var dx = x2 - x1;
    var dy = y2 - y1;
}
