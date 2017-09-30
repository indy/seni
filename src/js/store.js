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

import Util from './seni/Util';
import { SeniMode } from './ui/SeniMode';
import Job from './job';
import { jobBuildTraits,
         jobInitialGeneration,
         jobNewGeneration } from './jobTypes';

const logToConsole = true;

let currentState = undefined;

function actionSetMode(state, { mode }) {
  return new Promise((resolve, _reject) => {
    state.currentMode = mode;
    resolve(state);
  });
}

function actionSetScript(state, { script }) {
  return new Promise((resolve, reject) => {
    state.script = script;
    state.scriptHash = Util.hashCode(script);

    Job.request(jobBuildTraits, {
      script: state.script,
      scriptHash: state.scriptHash
    }).then(({ traits }) => {
      state.traits = traits;
      resolve(state);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function actionSetSelectedIndices(state, { selectedIndices }) {
  return new Promise((resolve, _reject) => {
    state.selectedIndices = selectedIndices || [];
    resolve(state);
  });
}

// todo: should populationSize be passed in the action?
function actionInitialGeneration(state) {
  return new Promise((resolve, reject) => {
    Job.request(jobInitialGeneration, {
      traits: state.traits,
      populationSize: state.populationSize
    }).then(({ genotypes }) => {
      state.genotypes = genotypes;
      state.previouslySelectedGenotypes = [];
      state.selectedIndices = [];
      resolve(state);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function actionShuffleGeneration(state, { rng }) {
  return new Promise((resolve, reject) => {
    const prev = state.previouslySelectedGenotypes;

    if (prev.length === 0) {
      actionInitialGeneration(state).then(s => {
        resolve(s);
      }).catch(error1 => {
        // handle error
        console.log(`worker: error of ${error1}`);
        reject(error1);
      });
    } else {
      Job.request(jobNewGeneration, {
        genotypes: prev,
        populationSize: state.populationSize,
        traits: state.traits,
        mutationRate: state.mutationRate,
        rng
      }).then(({ genotypes }) => {
        state.genotypes = genotypes;
        state.selectedIndices = [];
        resolve(state);
      }).catch(error => {
        // handle error
        console.log(`worker: error of ${error}`);
        reject(error);
      });
    }
  });
}

function actionNextGeneration(state, { rng }) {
  return new Promise((resolve, reject) => {
    const pg = state.genotypes;
    const selectedIndices = state.selectedIndices;
    const selectedGenos = [];
    for (let i = 0; i < selectedIndices.length; i++) {
      selectedGenos.push(pg[selectedIndices[i]]);
    }

    Job.request(jobNewGeneration, {
      genotypes: selectedGenos,
      populationSize: state.populationSize,
      traits: state.traits,
      mutationRate: state.mutationRate,
      rng
    }).then(({ genotypes }) => {
      const previouslySelectedGenotypes =
            genotypes.slice(0, selectedIndices.length);

      state.genotypes = genotypes;
      state.previouslySelectedGenotypes = previouslySelectedGenotypes;
      state.selectedIndices = [];

      resolve(state);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function wrapInPromise(state) {
  return new Promise((resolve, _reject) => {
    resolve(state);
  });
}

export function createInitialState() {
  return {
    // the resolution of the high res image
    highResolution: [2048, 2048],
    placeholder: 'img/spinner.gif',
    populationSize: 4,
    mutationRate: 0.1,

    currentMode: SeniMode.gallery,
    previouslySelectedGenotypes: [],
    selectedIndices: [],
    script: undefined,
    scriptHash: undefined,
    genotypes: [],
    traits: []
  };
}

function logMode(mode) {
  let name = '';
  switch(mode) {
  case SeniMode.gallery:
    name = 'gallery';
    break;
  case SeniMode.edit:
    name = 'edit';
    break;
  case SeniMode.evolve:
    name = 'evolve';
    break;
  default:
    name = 'unknown';
    break;
  }
  console.log(`SET_MODE: ${name}`);
}

export function createStore(initialState) {
  currentState = initialState;

  function reducer(state, action) {
    switch (action.type) {
    case 'SET_MODE':
      if (logToConsole) {
        logMode(action.mode);
      }
      return actionSetMode(state, action);
    case 'SET_SCRIPT':
      return actionSetScript(state, action);
    case 'SET_SELECTED_INDICES':
      return actionSetSelectedIndices(state, action);
    case 'INITIAL_GENERATION':
      return actionInitialGeneration(state);
    case 'NEXT_GENERATION':
      return actionNextGeneration(state, action);
    case 'SHUFFLE_GENERATION':
      return actionShuffleGeneration(state, action);
    case 'SET_STATE':
      if (logToConsole) {
        console.log(`SET_STATE: ${action.state}`);
      }
      return wrapInPromise(action.state);
    default:
      return wrapInPromise(state);
    }
  }

  function getState() {
    return currentState;
  }

  function dispatch(action) {
    if (logToConsole) {
      console.log(`dispatch: action = ${action.type}`);
    }
    return reducer(currentState, action);
  }

  return {
    getState,
    dispatch
  };
}
