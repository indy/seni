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
  // can't store the entire app since it contains DOM elements and there
  // is a 640k size limit on the serialized data structures.
  //
  const state = {
    currentMode: appState.get('currentMode'),
    previouslySelectedGenotypes:
    appState.get('previouslySelectedGenotypes').toJS(),
    selectedIndices: appState.get('selectedIndices').toJS(),
    script: appState.get('script'),
    genotypes: appState.get('genotypes').toJS()
  };

  const uri = `${seniModeAsString(appState.get('currentMode'))}`;
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
  return Immutable.fromJS(state);
}

export default {
  pushState,
  replaceState,
  restoreState
};
