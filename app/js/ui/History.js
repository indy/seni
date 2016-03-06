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

import { SeniMode } from './SeniMode';
import Immutable from 'immutable';

const logToConsole = false;

function seniModeAsString(mode) {
  switch (mode) {
  case SeniMode.gallery:
    return 'gallery';
  case SeniMode.edit:
    return 'edit';
  case SeniMode.evolve:
    return 'evolve';
  default:
    return 'error unknown SeniMode value';
  }
}

function buildState(appState) {
  const state = appState.toJS();
  const currentMode = appState.get('currentMode');
  const uri = `#${seniModeAsString(currentMode)}`;
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
  const allImmutable = Immutable.fromJS(state);

  // hack: state.traits is expected to consist of JS data structures
  // so convert that part of allImmutable back into JS
  //
  const immutableTraits = allImmutable.get('traits');
  return allImmutable.set('traits', immutableTraits.toJS());
}

export default {
  pushState,
  replaceState,
  restoreState
};
