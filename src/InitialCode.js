/*
    Seni
    Copyright (C) 2015  Inderjit Gill <email@indy.io>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

const code = `(wash)
(flower colour: (col/rgb r: [0.8 (scalar)]
                         g: [0.3 (scalar)]
                         b: [0.2 (scalar)]
                         alpha: [0.4 (scalar)])
        posx: 500 posy: 500 sc: [1.0 (scalar min: 0.6 max: 2.0)])

(define (petal-1
         angle: 0
         colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 alpha: 1.0))
  (on-matrix-stack
   (rotate angle: angle)
   (bezier-bulging tessellation: 20
                   line-width: [50 (int min: 10 max: 200)]
                   colour: colour
                   coords: (list (v2 0 0)
                                 (v2 233.33 100)
                                 (v2 566.66 -100)
                                 (v2 800 0)))))

(define (petal-2
         angle: 0
         colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 alpha: 1.0))
  (on-matrix-stack
   (rotate angle: angle)
   (bezier-bulging tessellation: 20
                   line-width: [50 (int min: 10 max: 200)]
                   colour: colour
                   coords: (list (v2 0 0)
                                 (v2 233.33 -100)
                                 (v2 566.66 100)
                                 (v2 800 0)))))

(define (circ-1
         petals: 0
         colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 alpha: 1.0)
         sc: 1.0)
  (on-matrix-stack
   (scale x: sc y: sc)
   (let ((strokes petals)
         (colcol (col/set-alpha colour: colour value: 0.3))
         (rem (remap-fn from: (list 0 strokes) to: (list 0 6.28318))))
     (loop (i from: 0 to: strokes)
           (petal-1 angle: (rem val: i) colour: colcol)))))

(define (circ-2
         petals: 0
         colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 alpha: 1.0)
         sc: 1.0)
  (on-matrix-stack
   (scale x: sc y: sc)
   (let ((strokes petals)
         (colcol (col/set-alpha colour: colour value: 0.4))
         (rem (remap-fn from: (list 0 strokes) to: (list 0 6.28318))))
     (loop (i from: 0 to: strokes)
           (petal-2 angle: (rem val: i) colour: colcol)))))

(define (layered-petals
         petals: 0
         colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 alpha: 1.0)
         sc: 1.0)
  (circ-1 petals: petals colour: colour sc: sc)
  (rotate angle: [0.1 (scalar min: 0.01 max: 0.4)])
  (circ-2 petals: petals colour: colour sc: sc))

(define (flower
         colour: (col/rgb r: 0.0 g: 1.0 b: 0.0 alpha: 0.5)
         posx: 0
         posy: 0
         sc: 1)
  (on-matrix-stack
   (translate x: posx y: posy)
   (scale x: sc y: sc)
   (let (((c2 c3) (col/analagous colour: colour)))
     (layered-petals petals: [23 (int min: 1 max: 50)] colour: colour sc: [0.6 (scalar min: 0.1 max: 0.9)])
     (layered-petals petals: [19 (int min: 1 max: 50)] colour: c3 sc: [0.5 (scalar min: 0.1 max: 0.9)])
     (layered-petals petals: [17 (int min: 1 max: 50)] colour: c2 sc: [0.3 (scalar min: 0.1 max: 0.9)]))))

(define (v
         x: 0
         y: 0
         z: 0
         scale: 1)
  (+ y (* scale (perlin/signed x: x y: y z: z))))

(define (wash variation: 200
              line-width: 70
              line-segments: 5
              colour: (col/rgb r: 0.627 g: 0.627 b: 0.627 alpha: 0.4)
              seed: 272)
  (loop (h from: -20 to: 1020 increment: 20)
        (bezier tessellation: line-segments
                line-width: line-width
                coords: (list
                         (v2 0 (v x: 0.10 y: h z: seed scale: variation))
                         (v2 333 (v x: 333.33 y: h z: seed scale: variation))
                         (v2 666 (v x: 666.66 y: h z: seed scale: variation))
                         (v2 1000 (v x: 1000.10 y: h z: seed scale: variation)))
                colour: colour)
        (bezier tessellation: line-segments
                line-width: line-width
                coords: (list
                         (v2 (v x: 0.10 y: h z: seed scale: variation) 0)
                         (v2 (v x: 333.33 y: h z: seed scale: variation) 333)
                         (v2 (v x: 666.66 y: h z: seed scale: variation) 666)
                         (v2 (v x: 1000.10 y: h z: seed scale: variation) 1000))
                colour: colour)))
`;

const InitialCode = {
  getCode: () => code
};

export default InitialCode;
