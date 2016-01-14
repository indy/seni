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

import { seniModeAsString } from './SeniMode';
import Immutable from 'immutable';

const logToConsole = false;

let jjj = 1;
function buildState(appState) {
  // can't store the entire app since it contains DOM elements and there
  // is a 640k size limit on the serialized data structures.
  //
  const state = {
    stateCounter: jjj,
    currentMode: appState.get('currentMode'),
    previouslySelectedGenotypes:
    appState.get('previouslySelectedGenotypes').toJS(),
    selectedIndices: appState.get('selectedIndices').toJS(),
    script: appState.get('script'),
    genotypes: appState.get('genotypes').toJS()
  };

  const uri = `#${seniModeAsString(appState.get('currentMode'))}-${jjj}`;
  jjj += 1;
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

  /**
   * Note: would like to use:
   *
   *    return Immutable.fromJS(state)
   *
   * but some of the genotypes may contain values that are plain JS arrays
   * e.g. seni code like:
   *
   * (define coords {[[10 10] [20 20] [20 20]] (vector)})
   *
   * don't want to convert them into Immutable objects as that will
   * screw up the later stages that expect plain JS objects/primitives
   */
  function deserializeGenotypes(genotypes) {
    return genotypes.reduce((list, genotype) => {
      const gt = genotype.reduce((lst, g) => lst.push(g), new Immutable.List());
      return list.push(gt);
    }, new Immutable.List());
  }

  return Immutable.fromJS({
    currentMode: state.currentMode,
    previouslySelectedGenotypes: deserializeGenotypes(
      state.previouslySelectedGenotypes),
    selectedIndices: state.selectedIndices,
    script: state.script,
    genotypes: deserializeGenotypes(state.genotypes)
  });
}

export default {
  pushState,
  replaceState,
  restoreState
};
