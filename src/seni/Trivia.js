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

// Trivia is a module that contains auxillery functions

function dayOfTheYear() {
  const now = new Date();
  const start = new Date(now.getFullYear(), 0, 0);
  const diff = now - start;

  const oneDay = 1000 * 60 * 60 * 24;

  return Math.floor(diff / oneDay);
}

function base10ToN(num, n) {

  const numRep = {10:'a', 11:'b', 12:'c', 13:'d', 14:'e', 15:'f', 16:'g',
                  17:'h', 18:'i', 19:'j', 20:'k', 21:'l', 22:'m', 23:'n',
                  24:'o', 25:'p', 26:'q', 27:'r', 28:'s', 29:'t', 30:'u',
                  31:'v', 32:'w', 33:'x', 34:'y', 35:'z'};

  let newNumString = '';
  let current = num;
  let remainderString, remainder;

  while (current !== 0) {
    remainder = current % n;
    if (36 > remainder && remainder > 9) {
      remainderString = numRep[remainder];
    } else if (remainder >= 36) {
      remainderString = '(' + remainder + ')';
    } else {
      remainderString = remainder;
    }
    newNumString = remainderString + newNumString;
    current = Number.parseInt(current / n);
  }

  return newNumString;
}

function _getTitle() {
  const now = new Date();
  const year = now.getYear() - 100; // years since 2000
  const d = base10ToN(dayOfTheYear(), 20);

  // in base 20 the days will have at most 2 digits,
  // so pad out the earlier one digit dates with 0
  const dayValue = ('00' + d).substr(-2);

  return '' + year + dayValue;
}

const Trivia = {
  getTitle: function() {
    return _getTitle();
  }
};

export default Trivia;
