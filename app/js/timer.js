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


const db = {};

function useDBEntry(id) {
  if (!db[id]) {
    db[id] = {
      id,
      values: []
    };
  }

  return db[id];
}

export function startTiming(id, konsole) {

  const entry = useDBEntry(id);

  const stopFn = () => {
    const after = new Date();
    const duration = after - before;

    entry.values.push(duration);
    if (konsole) {
      konsole.log(`rendered ${entry.id} ${duration}ms`);
    }
  };

  const before = new Date();
  return stopFn;
}

export function getTimingEntry(id) {
  return db[id];
}
