/*
 *  Seni
 *  Copyright (C) 2015 Inderjit Gill <email@indy.io>
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

const Util = {
  merge: (obj, defaults) => {
    const res = {};
    for (const p in defaults) {
      res[p] = obj[p] !== undefined ? obj[p] : defaults[p];
    }
    return res;
  },

  // execute the function and log the time that it takes
  withTiming: (msg, fn, shouldLog) => {
    const before = new Date();
    fn();
    const after = new Date();
    const duration = after - before;
    if (shouldLog) {
      console.log(msg, duration, 'ms');
    }
  }
};

export default Util;
