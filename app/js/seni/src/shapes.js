import { PublicBinding } from 'lang/env';
import { Renderer } from './Renderer';
import { GLContainer } from './GLContainer';
import { Buffer } from './Buffer';
import {
  normalize,
  stepsInclusive,
  remapFn
} from './MathUtil';
import {Colour, Format} from './Colour';


export const rect = new PublicBinding(
  "rect",
  `
  this fn adds 1
  this is a multi line comment
  woo hoo
  `,
  (renderer) => {
    const glContainer = renderer.getGLContainer();
    const buffer = renderer.getBuffer();

    // return a function which accepts args as parameters
    return ({ x = 0,
              y = 0,
              width = 100,
              height = 100,
              colour = undefined}) => {
                const halfWidth = (width / 2);
                const halfHeight = (height / 2);


                //console.log("rect: x:" + x + ", y:" + y + ", width:" + width + ", height:" + height);
                
                if(colour === undefined) {
                  colour = new Colour(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
                } else if(colour.format !== Format.RGB) {
                  colour = colour.cloneAs(Format.RGB);
                }

                buffer.prepareToAddTriangleStrip(glContainer, 4,
                                                 [x - halfWidth, y - halfHeight, 0.0]);
                buffer.addVertex([x - halfWidth, y - halfHeight, 0.0], colour.val);
                buffer.addVertex([x + halfWidth, y - halfHeight, 0.0], colour.val);
                buffer.addVertex([x - halfWidth, y + halfHeight, 0.0], colour.val);
                buffer.addVertex([x + halfWidth, y + halfHeight, 0.0], colour.val);
              }
  })

export const bezier = new PublicBinding(
  "bezier",
  `
  this fn adds 1
  this is a multi line comment
  woo hoo
  `,
  (renderer) => {
    const glContainer = renderer.getGLContainer();
    const buffer = renderer.getBuffer();

    return ({tessellation = 15,
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
             colour = undefined}) => {

               let halfWidthEnd, remap;

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
                 colour = new Colour(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
               } else if(colour.format !== Format.RGB) {
                 colour = colour.cloneAs(Format.RGB);
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
  })

export const bezierTrailing = new PublicBinding(
  "bezierTrailing",
  `
  this fn adds 1
  this is a multi line comment
  woo hoo
  `,
  (renderer) => {

    let bezierFn = bezier.create(renderer);

    // return a function which accepts args as parameters
    return ({ tessellation = 15,
              lineWidth = 20,
              coords = [[440, 400],
                        [533, 700],
                        [766, 200],
                        [900, 500]],
              tStart = 0,
              tEnd = 1,
              colour = undefined}) => {
                bezierFn({tessellation: tessellation,
                          lineWidthStart: lineWidth,
                          lineWidthEnd: 0.0,
                          lineWidthMapping: "linear",
                          coords: coords,
                          tStart: tStart,
                          tEnd: tEnd,
                          colour: colour})
              }
  })

export const bezierBulging = new PublicBinding(
  "bezierBulging",
  `
  this fn adds 1
  this is a multi line comment
  woo hoo
  `,
  (renderer) => {

    let bezierFn = bezier.create(renderer);

    // return a function which accepts args as parameters
    return ({ tessellation = 16,
              lineWidth = 20,
              coords = [[440, 400],
                        [533, 700],
                        [766, 200],
                        [900, 500]],
              tStart = 0,
              tEnd = 1,
              colour = undefined}) => {

  let tMid = (tStart + tEnd) / 2.0;
  // tessellation should be an even number
  let newTess = tessellation & 1 ? tessellation + 1: tessellation;

  //let red = new Colour(Format.RGB, [1.0, 0.0, 0.0, 1.0]);
  //renderRect(renderer, {x: coords[0][0], y: coords[0][1], width: 20, height: 20, colour: red});
  //renderRect(renderer, {x: coords[3][0], y: coords[3][1], width: 20, height: 20, colour: red});

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
              }
  })

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
