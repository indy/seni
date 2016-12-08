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
  'brushA': [makeMapping(0, 781, 976, 1023)]
};


export default {
  get(type, subtype) {
    return mapping[type][subtype];
  },

  uv(x, y) {
    return uv(x, y);
  }
};
