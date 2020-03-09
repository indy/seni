/*
 *  Copyright (C) 2020 Inderjit Gill <email@indy.io>
 *
 *  This file is part of Seni
 *
 *  Seni is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Affero General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Seni is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU Affero General Public License for more details.
 *
 *  You should have received a copy of the GNU Affero General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

namespace Log {
    export let logToConsole: boolean = true;

    export function log(msg: string) {
        if (logToConsole) {
            console.log(msg);
        }
    }

    export function error(msg: string) {
        console.error(msg);
    }

    export function debug(obj: any) {
        console.debug(obj);
    }
}
