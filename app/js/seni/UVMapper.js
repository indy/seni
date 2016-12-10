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

const brushInfo = {
  'flat': [
    {
      mapping: makeMapping(1, 1, 2, 2),
      widthScale: 1.0
    }
  ],
  'brushA': [
    {
      mapping: makeMapping(0, 781, 976, 1023),
      widthScale: 1.2
    }
  ],
  'brushB': [
    {
      mapping: makeMapping(11, 644, 490, 782),
      widthScale: 1.4
    },
    {
      mapping: makeMapping(521, 621, 1023, 783),
      widthScale: 1.1
    },
    {
      mapping: makeMapping(340, 419, 666, 508),
      widthScale: 1.2
    },
    {
      mapping: makeMapping(326, 519, 659, 608),
      widthScale: 1.2
    },
    {
      mapping: makeMapping(680, 419, 1020, 507),
      widthScale: 1.1
    },
    {
      mapping: makeMapping(677, 519, 1003, 607),
      widthScale: 1.1
    }
  ],
  'brushC': [
    {
      mapping: makeMapping(0, 7, 324, 43),
      widthScale: 1.2
    },
    {
      mapping: makeMapping(0, 45, 319, 114),
      widthScale: 1.3
    },
    {
      mapping: makeMapping(0, 118, 328, 180),
      widthScale: 1.1
    },
    {
      mapping: makeMapping(0, 186, 319, 267),
      widthScale: 1.2
    },
    {
      mapping: makeMapping(0, 271, 315, 334),
      widthScale: 1.4
    },
    {
      mapping: makeMapping(0, 339, 330, 394),
      widthScale: 1.1
    },
    {
      mapping: makeMapping(0, 398, 331, 473),
      widthScale: 1.2
    },
    {
      mapping: makeMapping(0, 478, 321, 548),
      widthScale: 1.1
    },
    {
      mapping: makeMapping(0, 556, 326, 618),
      widthScale: 1.1
    }
  ],
  'brushD': [
    {
      mapping: makeMapping(333, 165, 734, 336),
      widthScale: 1.3
    }
  ],
  'brushE': [
    {
      mapping: makeMapping(737, 183, 1018, 397),
      widthScale: 1.3
    }
  ],
  'brushF': [
    {
      mapping: makeMapping(717, 2, 1023, 163),
      widthScale: 1.1
    }
  ],
  'brushG': [
    {
      mapping: makeMapping(329, 0, 652, 64),
      widthScale: 1.2
    },
    {
      mapping: makeMapping(345, 75, 686, 140),
      widthScale: 1.0
    }
  ]
};


export default {
  get(type, subtype) {
    return brushInfo[type][subtype];
  },

  uv(x, y) {
    return uv(x, y);
  }
};
