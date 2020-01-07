/*
 *  Seni
 *  Copyright (C) 2019 Inderjit Gill <email@indy.io>
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

///<reference path='Log.ts'/>

namespace Timer {

    const db:any = {};
    const printPrecision = 2;

    export class DbEntry {
        id: string;
        num: number;
        sum: number;
        last: number;
        min: number;
        max: number;

        constructor(id: string) {
            this.id = id;
            this.num = 0;
            this.sum = 0;
            this.last = 0;
            this.min = 100000;
            this.max = 0;
        }
    }

    export class Stats {
        current: number;
        average: number;
        min: number;
        max: number;
        num: number;

        constructor(entry: DbEntry) {
            this.current = entry.last;
            this.average = (entry.sum / entry.num);
            this.min = entry.min;
            this.max = entry.max;
            this.num = entry.num;
        }
    }

    export function getStats(entry: DbEntry): Stats | null {
        if (entry.num === 0) {
            return null;
        }

        return new Stats(entry);
    }


    export function addTiming(entry: DbEntry, duration: number): DbEntry {
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

    export function useDBEntry(id: string): DbEntry {
        if (!db[id]) {
            db[id] = new DbEntry(id);
        }

        return db[id];
    }

    export function startTiming(): (arg0: string) => void {
        if (Log.logToConsole) {
            const before = performance.now();
            // return the 'stop' function
            return (id: string) => {
                const entry = useDBEntry(id);

                const after = performance.now();
                const duration = after - before;

                addTiming(entry, duration);

                const stats = getStats(entry);

                if (stats) {
                    const eid = entry.id;
                    const cur = stats.current.toFixed(printPrecision);
                    const avg = stats.average.toFixed(printPrecision);
                    const min = stats.min.toFixed(printPrecision);
                    const max = stats.max.toFixed(printPrecision);
                    const num = stats.num;

                    const msg1 = `${eid}: ${cur}ms `;
                    const msg2 = `(Mean: ${avg}, Min: ${min}, Max: ${max} N:${num})`;

                    Log.log(msg1 + msg2);
                }
            };
        } else {
            // do nothing
            return (_id: string) => {};
        }
    }

    export function getTimingEntry(id: string): DbEntry {
        return db[id];
    }
}
