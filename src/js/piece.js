/*
 *  Sen
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

import main from './app-piece.js';

function compatibilityHacks() {
  // Safari doesn't have Number.parseInt (yet)
  // Safari is the new IE
  if (Number.parseInt === undefined) {
    Number.parseInt = parseInt;
  }
}

document.addEventListener('DOMContentLoaded', () => {
  compatibilityHacks();
  main();
});
