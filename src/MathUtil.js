


// are these global?
var PI = Math.PI;
var twoPI = PI * 2;
var PIbyTwo = PI / 2;

export function stepsInclusive(start, end, num) {
    var unit = (end - start) / (num - 1);
    var res = [];
    for(var i=0;i<num;i++) {
        res.push(start + (i * unit));
    }
    return res;
}

export function clamp(a, min, max) {
    return a < min ? min : (a > max ? max : a);
}

export function distance1d(a, b) {
    return Math.abs(a - b);
}

export function distance2d([xa, ya], [xb, yb]) {
    let xd = xa - xb;
    let yd = ya - yb;
    return Math.sqrt((xd * xd) + (yd * yd));
}

export function normalize(x, y) {
    let len = Math.sqrt((x * x) + (y * y));
    return [(x / len), (y / len)];
}

export function mc([xa, ya], [xb, yb]) {
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
    let s = Math.sin(x * PIbyTwo);
    return s * s * s * s;
}

function mapSlowEaseInEaseOut(x) {
    return x - (Math.sin(x * twoPI) / twoPI);
}

var mappingLookup = new Map([["linear", mapLinear],
                             ["quick", mapQuickEase],
                             ["slow-in", mapSlowEaseIn],
                             ["slow-in-out", mapSlowEaseInEaseOut]]);

export function remapFn({from = [0, 1],
                         to = [0, 100],
                         clamping = false,
                         mapping = "linear"}) {

    let [fromA, fromB] = from,
    [toA, toB] = to,
    [fromM, fromC] = mc([fromA, 0], [fromB, 1]),
    [toM, toC] = mc([0, toA], [1, toB]),
    normalisedMappingFn = mappingLookup.get(mapping);

    if(normalisedMappingFn === undefined) {
        // console.log("unknown mapping method given to remapFn: " + mapping
        normalisedMappingFn = mappingLookup.get("linear");
    }

    return function(x) {
        let fromInterp = (fromM * x) + fromC,
        toInterp = normalisedMappingFn(fromInterp),
        res = (toM * toInterp) + toC;
        if(clamping) {
            return fromInterp < 0 ? toA : (fromInterp > 1) ? toB : res;
        } else {
            return res;
        }
    };
}
