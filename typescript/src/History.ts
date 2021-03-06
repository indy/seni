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

///<reference path='Log.ts'/>
///<reference path='Controller.ts'/>
///<reference path='State.ts'/>

namespace SeniHistory {
    function senModeAsString(state: State): string {
        const mode = state.currentMode;

        switch (mode) {
            case SeniMode.gallery:
                return 'gallery';
            case SeniMode.edit:
                if (state.scriptId) {
                    return state.scriptId.toString();
                } else {
                    return "Error: currentMode is edit but there is no state.scriptId?";
                }
            case SeniMode.evolve:
                return 'evolve';
            default:
                return 'error unknown SeniMode value';
        }
    }

    function buildState(appState: State): [State, string] {
        const state = appState;
        const uri = `#${senModeAsString(state)}`;

        return [state, uri];
    }

    export function pushState(appState: State) {
        const [state, uri] = buildState(appState);
        Log.log('historyPushState');
        history.pushState(state, "", uri);
    }

    export function replaceState(appState: State) {
        const [state, uri] = buildState(appState);
        Log.log('historyReplace');
        history.replaceState(state, "", uri);
    }

    export function restoreState(stateObj: object) {
        Log.log('historyRestore');
        // note: the popstate event on window returns a plain js object.
        // this needs to be converted back into a State instance
        //
        const state = State.createStateFromObject(<State>stateObj);

        return state;
    }
}
