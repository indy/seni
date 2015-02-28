import PublicBinding from '../lang/PublicBinding';
import SeedRandom from './SeedRandom';

var _PI = Math.PI;
var _twoPI = _PI * 2;
var _PIbyTwo = _PI / 2;

function mc([xa, ya], [xb, yb]) {
  let m = (ya - yb) / (xa - xb);
  let c = ya - (m * xa);
  return [m, c];
}

// the following map* functions work in the range 0..1:

function mapLinear(x) {
  return x;
}

function mapQuickEase(x) {
  let x2 = x * x;
  let x3 = x * x * x;
  return (3 * x2) - (2 * x3);
}

function mapSlowEaseIn(x) {
  let s = Math.sin(x * _PIbyTwo);
  return s * s * s * s;
}

function mapSlowEaseInEaseOut(x) {
  return x - (Math.sin(x * _twoPI) / _twoPI);
}

var mappingLookup = new Map([['linear', mapLinear],
                             ['quick', mapQuickEase],
                             ['slow-in', mapSlowEaseIn],
                             ['slow-in-out', mapSlowEaseInEaseOut]]);

function _remapFn(params) {

  let from = params.from || [0, 1];
  let to = params.to || [0, 100];
  let clamping = params.clamping || false;
  let mapping = params.mapping || 'linear';

  let [fromA, fromB] = from,
      [toA, toB] = to,
      [fromM, fromC] = mc([fromA, 0], [fromB, 1]),
      [toM, toC] = mc([0, toA], [1, toB]),
      normalisedMappingFn = mappingLookup.get(mapping);

  if(normalisedMappingFn === undefined) {
    normalisedMappingFn = mappingLookup.get('linear');
  }

  return function(params) {
    let val = params.val || 0;
    let fromInterp = (fromM * val) + fromC,
        toInterp = normalisedMappingFn(fromInterp),
        res = (toM * toInterp) + toC;
    if(clamping) {
      return fromInterp < 0 ? toA : (fromInterp > 1) ? toB : res;
    } else {
      return res;
    }
  };
}


var MathUtil = {

  remapFn: _remapFn,

  PI: new PublicBinding(
    'PI',
    ``,
    () => _PI
  ),

  twoPI: new PublicBinding(
    'twoPI',
    ``,
    () => _twoPI
  ),

  PIbyTwo: new PublicBinding(
    'PIbyTwo',
    ``,
    () => _PIbyTwo
  ),

  remapFnBinding: new PublicBinding(
    'remapFn',
    ``,
    () => _remapFn
  ),

  buildPRNG: new PublicBinding(
    'buildPRNG',
    `returns a function that generates a prng everytime its invoked`,

    () => function (params) {
      let seed = params.seed || 'hello.';
      return SeedRandom.buildPRNG(seed);
    }
  ),

  stepsInclusive: function(start, end, num) {
    var unit = (end - start) / (num - 1);
    var res = [];
    for(var i=0;i<num;i++) {
      res.push(start + (i * unit));
    }
    return res;
  },

  clamp: function(a, min, max) {
    return a < min ? min : (a > max ? max : a);
  },

  distance1d: function(a, b) {
    return Math.abs(a - b);
  },

  distance2d: function([xa, ya], [xb, yb]) {
    let xd = xa - xb;
    let yd = ya - yb;
    return Math.sqrt((xd * xd) + (yd * yd));
  },

  normalize: function(x, y) {
    let len = Math.sqrt((x * x) + (y * y));
    return [(x / len), (y / len)];
  },

  mc: function([xa, ya], [xb, yb]) {
    let m = (ya - yb) / (xa - xb);
    let c = ya - (m * xa);
    return [m, c];
  }
};

export default MathUtil;

