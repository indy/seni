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


const db = {};
const printPrecision = 2;

function getStats(entry) {
  if (entry.num === 0) {
    return null;
  }
  return {
    current: entry.last,
    average: (entry.sum / entry.num),
    min: entry.min,
    max: entry.max,
    num: entry.num
  };
}


function addTiming(entry, duration) {
  entry.num += 1;
  entry.sum += duration;
  entry.last = duration;
  if (duration < entry.min) {
    entry.min = duration;
  }
  if (duration > entry.max) {
    entry.max = duration;
  }
  return entry;
}

function useDBEntry(id) {
  if (!db[id]) {
    db[id] = {
      id,
      num: 0,
      sum: 0,
      last: 0,
      min: 100000,
      max: 0
    };
  }

  return db[id];
}

export function startTiming() {
  const before = performance.now();
  // return the 'stop' function
  return (id, konsole) => {
    const entry = useDBEntry(id);

    const after = performance.now();
    const duration = after - before;

    addTiming(entry, duration);

    const stats = getStats(entry);

    if (konsole && stats) {
      const eid = entry.id;
      const cur = stats.current.toFixed(printPrecision);
      const avg = stats.average.toFixed(printPrecision);
      const min = stats.min.toFixed(printPrecision);
      const max = stats.max.toFixed(printPrecision);
      const num = stats.num;

      const msg1 = `${eid}: ${cur}ms `;
      const msg2 = `(Mean: ${avg}, Min: ${min}, Max: ${max} N:${num})`;

      konsole.log(msg1 + msg2);
    }
  };
}

export function getTimingEntry(id) {
  return db[id];
}
