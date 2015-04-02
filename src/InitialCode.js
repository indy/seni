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

(define coords (pair 440 400
               533 700
                   766 200
                   900 500))

(define coords2 (pair 440 700
                      533 1000
                      766 500
                      900 800))

(define coords3 (pair 440 800
                      533 1100
                      766 600
                      900 900))

(spline colour: (col/rgb r: 0.0 g: 0.4 b: 0.1 a: 0.5)
        lineWidthStart:10
        lineWidthEnd: 0
        coords: (pair 140 400
                      333 600
                      566 500))

(bezier tessellation: 35
    lineWidth: 10
     coords: coords
     colour: (col/rgb r: 0.0 g: 0.0 b: 0.1 a: 0.5))

(strokedBezier tessellation: 5
               coords: coords2
               strokeLineWidthStart: 20
               strokeLineWidthEnd: 20
               strokeTessellation: 15
               strokeNoise: 25
               colour: (col/rgb r: 0.2 g: 0.9 b: 0.1 a: 0.3)
               colourVolatility: 0
 seed: 43)

(strokedBezier tessellation: 15
               coords: coords3
               strokeLineWidthStart: 20
               strokeLineWidthEnd: 4
               strokeTessellation: 15
               strokeNoise: 45
               colour: (col/rgb r: 0.2 g: 0.9 b: 0.1 a: 0.3)
               colourVolatility: 70
               seed: 2200)

(strokedBezier tessellation: 65
               coords: coords3
               strokeLineWidthStart: 2
               strokeLineWidthEnd: 5
               strokeTessellation: 15
               strokeNoise: 55
               colour: (col/rgb r: 0.2 g: 0.9 b: 0.1 a: 0.3)
               colourVolatility: 700
               seed: 220)

(strokedBezierRect x: 50
                   y: 50
                   width: 400
                   height: 400

                   iterations: 50
                   overlap: 10

                   tessellation: 35

                   strokeTessellation: 15
                   strokeNoise: 90

                   colour: (col/rgb r: 0.2 g: 0.0 b: 0.1 a: 0.9)
                   colourVolatility: 20

                   volatility: 17
                   seed: 43)
`;

const InitialCode = {
  getCode: () => code
};

export default InitialCode;
