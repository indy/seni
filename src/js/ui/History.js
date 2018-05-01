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

import { SenMode } from './SenMode';

const logToConsole = true;

function senModeAsString(mode) {
  switch (mode) {
  case SenMode.gallery:
    return 'gallery';
  case SenMode.edit:
    return 'edit';
  case SenMode.evolve:
    return 'evolve';
  default:
    return 'error unknown SenMode value';
  }
}

function buildState(appState) {
  const state = appState;
  const currentMode = appState.currentMode;
  const uri = `#${senModeAsString(currentMode)}`;
  return [state, uri];
}

function pushState(appState) {
  const [state, uri] = buildState(appState);
  if (logToConsole) {
    console.log('historyPushState', state);
  }
  history.pushState(state, null, uri);
}

function replaceState(appState) {
  const [state, uri] = buildState(appState);
  if (logToConsole) {
    console.log('historyReplace', state);
  }
  history.replaceState(state, null, uri);
}

function restoreState(state) {
  if (logToConsole) {
    console.log('historyRestore', state);
  }

  return state;
}

export default {
  pushState,
  replaceState,
  restoreState
};
