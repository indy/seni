/*
 *  Seni
 *  Copyright (C) 2016 Inderjit Gill <email@indy.io>
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

const textureDim = 1024.0;
function uv(x, y) {
  return [x / textureDim, y / textureDim];
}

// convert texture x,y locations into uv mappings
//
function makeMapping(minX, minY, maxX, maxY) {
  return [uv(maxX, minY),
          uv(maxX, maxY),
          uv(minX, minY),
          uv(minX, maxY)
         ];
}

const mapping = {
  'flat': [makeMapping(1, 1, 2, 2)],
  'brushA': [makeMapping(0, 781, 976, 1023)],
  'brushB': [makeMapping(11, 644, 490, 782),
             makeMapping(521, 621, 1023, 783),
             makeMapping(340, 419, 666, 508),
             makeMapping(326, 519, 659, 608),
             makeMapping(680, 419, 1020, 507),
             makeMapping(677, 519, 1003, 607)
            ],
  'brushC': [makeMapping(0, 7, 324, 43),
             makeMapping(0, 45, 319, 114),
             makeMapping(0, 118, 328, 180),
             makeMapping(0, 186, 319, 267),
             makeMapping(0, 271, 315, 334),
             makeMapping(0, 339, 330, 394),
             makeMapping(0, 398, 331, 473),
             makeMapping(0, 478, 321, 548),
             makeMapping(0, 556, 326, 618)
            ],
  'brushD': [makeMapping(333, 165, 734, 336)],
  'brushE': [makeMapping(737, 183, 1018, 397)],
  'brushF': [makeMapping(717, 2, 1023, 163)],
  'brushG': [makeMapping(329, 0, 652, 64),
             makeMapping(345, 75, 686, 140)]
};


export default {
  get(type, subtype) {
    return mapping[type][subtype];
  },

  uv(x, y) {
    return uv(x, y);
  }
};
