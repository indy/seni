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

export default {
  // from http://werxltd.com/wp/2010/05/13/ (cont'd next line)
  // javascript-implementation-of-javas-string-hashcode-method/
  hashCode: string => {
    let hash = 0, i, chr, len;
    if (string.length === 0) return hash;
    for (i = 0, len = string.length; i < len; i++) {
      chr = string.charCodeAt(i);
      hash = ((hash << 5) - hash) + chr;
      hash |= 0; // Convert to 32bit integer
    }
    return hash;
  },

  /**
   * Execute the function and log the time that it takes
   *
   * @param {string}   msg     the message to log
   * @param {function} fn      the function to time
   * @param {Object}   console the object that can log the timing information
   */
  //
  withTiming: (msg, fn, console) => {
    const before = new Date();
    fn();
    const after = new Date();
    const duration = after - before;
    if (console) {
      console.log(`${msg} ${duration} ms`);
    }
  }
};
