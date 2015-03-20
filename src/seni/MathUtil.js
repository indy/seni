import PublicBinding from '../lang/PublicBinding';
import SeedRandom from './SeedRandom';

const _PI = Math.PI;
const _twoPI = _PI * 2;
const _PIbyTwo = _PI / 2;

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

const mappingLookup = new Immutable.Map({'linear': mapLinear,
                                       'quick': mapQuickEase,
                                       'slow-in': mapSlowEaseIn,
                                       'slow-in-out': mapSlowEaseInEaseOut});

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


const MathUtil = {

  remapFn: _remapFn,

  PI: new PublicBinding(
    'PI',
    ``,
    {},
    () => _PI
  ),

  twoPI: new PublicBinding(
    'twoPI',
    ``,
    {},
    () => _twoPI
  ),

  PIbyTwo: new PublicBinding(
    'PIbyTwo',
    ``,
    {},
    () => _PIbyTwo
  ),

  remapFnBinding: new PublicBinding(
    'remapFn',
    ``,
    {},
    () => _remapFn
  ),

  rngUnsigned: new PublicBinding(
    'rng/unsigned',
    `returns a function that generates a random number in the range 0..1 everytime its invoked`,
    {seed: 'shabba'},
    (self) => function (params) {
      let {seed} = self.mergeWithDefaults(params);
      return SeedRandom.buildUnsigned(seed);
    }
  ),

  rngSigned: new PublicBinding(
    'rng/signed',
    `returns a function that generates a random number in the range -1..1 everytime its invoked`,
    {seed: 'shabba'},
    (self) => function (params) {
      let {seed} = self.mergeWithDefaults(params);
      return SeedRandom.buildSigned(seed);
    }
  ),

  distance2D: new PublicBinding(
    'math/distance2D',
    ``,
    {aX: 0, aY: 0, bX: 1, bY: 1},
    (self) => function (params) {
      let {aX, aY, bX, bY} = self.mergeWithDefaults(params);

      const xd = aX - bX;
      const yd = aY - bY;
      return Math.sqrt((xd * xd) + (yd * yd));
    }
  ),

  clamp: new PublicBinding(
    'math/clamp',
    ``,
    {val: 0, min: 0, max: 1},
    (self) => function (params) {
      let {val, min, max} = self.mergeWithDefaults(params);
      return val < min ? min : (val > max ? max : val);
    }
  ),

  stepsInclusive: function(start, end, num) {
    const unit = (end - start) / (num - 1);
    let res = [];
    for(let i=0;i<num;i++) {
      res.push(start + (i * unit));
    }
    return res;
  },

  distance1d: function(a, b) {
    return Math.abs(a - b);
  },
/*
  clamp: function(a, min, max) {
 return a < min ? min : (a > max ? max : a);
  },

  distance2d: function([xa, ya], [xb, yb]) {
    let xd = xa - xb;
    let yd = ya - yb;
    return Math.sqrt((xd * xd) + (yd * yd));
  },
 */

  interpolate: function(a, b, t) {
    return (a * (1 - t)) + (b * t);
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
