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

(define centre (v2 500 500))

(define (render-circle position: (v2 0 0)
                       step: 0
                       t-value: 0)
  (let ((rad (- 400 (* step 4))))
    (poly position: (v2/+ centre (v2/* position (v2 rad rad)))
          radius: (- 30 step)
          tessellation: 30
          colour: (col/rgb r: 0.1 g: 0.5 b: 0.2 alpha: 0.7))))

(path/circle position: (v2 0 0)
             radius: 1
             steps: 20
             fn: render-circle)

`;

const InitialCode = {
  getCode: () => code
};

export default InitialCode;
