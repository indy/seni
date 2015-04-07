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

const code = `

(define backgroundColour (col/rgb r: [0.2 (scalar)]
                                  g: [0.2 (scalar)]
                                  b: [0.5 (scalar)]
                                  alpha: 0.9))

(define topColour (col/rgb r: [0.2 (scalar)]
                           g: [0.0 (scalar)]
                           b: [0.1 (scalar)]
                           alpha: 0.9))

(define midColour (col/lab l: [37.0 (scalar min: 0 max: 100)]
                           a: [5.79 (scalar min: -50 max: 50)]
                           b: [-35.82 (scalar min: -50 max: 50)]
                           alpha: 0.7))

(define lowColour (col/set-lab-l colour: midColour
                                 value: (- (col/get-lab-l colour: midColour)
                                           [20.5 (scalar min: 0.0 max: 20.0)])))

(define topHi (col/set-lab-l colour: topColour
                             value: (+ (col/get-lab-l colour: topColour)
                                       [10.5 (scalar min: 0.0 max: 20.0)])))

; single colour background
(rect x: 500
      y: 500
      width: 1000
      height: 1000
      colour: backgroundColour)

; the textured background
(strokedBezierRect x: 500
                   y: 500
                   width: 1000
                   height: 1000

                   iterations: [100 (int min: 1 max: 100)]
                   overlap: [10 (int min: 1 max: 100)]

                   tessellation: [35 (int min: 1 max: 50)]

                   strokeTessellation: 15
                   strokeNoise: [30 (int min: 1 max: 50)]

                   colour: backgroundColour
                   colourVolatility: [20 (int min: 0 max: 30)]

                   volatility: [3 (int min: 0 max: 20)]
                   seed: [44 (int)])
; top section
(strokedBezierRect x: 500
                   y: 750
                   width: 900
                   height: 400

                   iterations: [40 (int min: 1 max: 100)]
                   overlap: [10 (int min: 1 max: 100)]

                   tessellation: [35 (int min: 1 max: 50)]

                   strokeTessellation: 15
                   strokeNoise: [20 (int min: 1 max: 50)]

                   colour: topColour
                   colourVolatility: [10 (int min: 0 max: 30)]

                   volatility: [10 (int min: 0 max: 30)]
                   seed: [44 (int)])

; top section highlights

(define hiIterations [5 (int min: 1 max: 30)])
(define hiOverlap [1 (int min: 1 max: 20)])

(strokedBezierRect x: 250
                   y: 750
                   width: 200
                   height: 300

                   iterations: hiIterations
                   overlap: hiOverlap

                   tessellation: [35 (int min: 1 max: 50)]

                   strokeTessellation: 15
                   strokeNoise: [20 (int min: 1 max: 50)]

                   colour: topHi
                   colourVolatility: [10 (int min: 0 max: 30)]

                   volatility: [10 (int min: 0 max: 30)]
                   seed: [44 (int)])

(strokedBezierRect x: 550
                   y: 750
                   width: 200
                   height: 300

                   iterations: hiIterations
                   overlap: hiOverlap

                   tessellation: [35 (int min: 1 max: 50)]

                   strokeTessellation: 15
                   strokeNoise: [20 (int min: 1 max: 50)]

                   colour: topHi
                   colourVolatility: [10 (int min: 0 max: 30)]

                   volatility: [10 (int min: 0 max: 30)]
                   seed: [44 (int)])


(strokedBezierRect x: 850
                   y: 750
                   width: 200
                   height: 300

                   iterations: hiIterations
                   overlap: hiOverlap

                   tessellation: [35 (int min: 1 max: 50)]

                   strokeTessellation: 15
                   strokeNoise: [20 (int min: 1 max: 50)]

                   colour: topHi
                   colourVolatility: [10 (int min: 0 max: 30)]

                   volatility: [10 (int min: 0 max: 30)]
                   seed: [44 (int)])

; middle section
(strokedBezierRect x: 500
                   y: 425
                   width: 900
                   height: 250

                   iterations: [40 (int min: 1 max: 100)]
                   overlap: [10 (int min: 1 max: 100)]

                   tessellation: [35 (int min: 1 max: 50)]

                   strokeTessellation: 15
                   strokeNoise: [20 (int min: 1 max: 50)]

                   colour: midColour
                   colourVolatility: [10 (int min: 0 max: 30)]

                   volatility: [10 (int min: 0 max: 30)]
                   seed: [44 (int)])

; lower section
(strokedBezierRect x: 500
                   y: 200
                   width: 900
                   height: 300

                   iterations: [40 (int min: 1 max: 100)]
                   overlap: [10 (int min: 1 max: 100)]

                   tessellation: [35 (int min: 1 max: 50)]

                   strokeTessellation: 15
                   strokeNoise: [20 (int min: 1 max: 50)]

                   colour: lowColour
                   colourVolatility: [10 (int min: 0 max: 30)]

                   volatility: [10 (int min: 0 max: 30)]
                   seed: [44 (int)])

`;

const InitialCode = {
  getCode: () => code
};

export default InitialCode;
