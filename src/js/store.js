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

function cloneState(state) {
  const clone = {};

  clone.highResolution = state.highResolution;
  clone.placeholder = state.placeholder;
  clone.populationSize = state.populationSize;
  clone.mutationRate = state.mutationRate;

  clone.currentMode = state.currentMode;
  clone.galleryLoaded = state.galleryLoaded;
  clone.previouslySelectedGenotypes = state.previouslySelectedGenotypes;
  clone.selectedIndices = state.selectedIndices;
  clone.scriptId = state.scriptId;
  clone.script = state.script;
  clone.scriptHash = state.scriptHash;
  clone.genotypes = state.genotypes;
  clone.traits = state.traits;

  return clone;
}

function resolveAsCurrentState(resolve, state) {
  currentState = state;
  resolve(currentState);
}

function actionSetMode(state, { mode }) {
  return new Promise((resolve, _reject) => {
    const newState = cloneState(state);
    newState.currentMode = mode;
    resolveAsCurrentState(resolve, newState);
  });
}

function actionSetScript(state, { script }) {
  return new Promise((resolve, reject) => {
    const newState = cloneState(state);
    newState.script = script;
    newState.scriptHash = Util.hashCode(script);

    Job.request(jobBuildTraits, {
      script: newState.script,
      scriptHash: newState.scriptHash
    }).then(({ traits }) => {
      newState.traits = traits;
      resolveAsCurrentState(resolve, newState);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function actionSetScriptId(state, { id }) {
  console.log(`actionSetScriptId id: ${id}`);

  return new Promise((resolve, _reject) => {
    const newState = cloneState(state);
    newState.scriptId = id;
    resolveAsCurrentState(resolve, newState);
  });
}

function actionSetSelectedIndices(state, { selectedIndices }) {
  return new Promise((resolve, _reject) => {
    const newState = cloneState(state);
    newState.selectedIndices = selectedIndices || [];
    resolveAsCurrentState(resolve, newState);
  });
}

// todo: should populationSize be passed in the action?
function actionInitialGeneration(state) {
  return new Promise((resolve, reject) => {
    const newState = cloneState(state);
    Job.request(jobInitialGeneration, {
      traits: newState.traits,
      populationSize: newState.populationSize
    }).then(({ genotypes }) => {
      newState.genotypes = genotypes;
      newState.previouslySelectedGenotypes = [];
      newState.selectedIndices = [];
      resolveAsCurrentState(resolve, newState);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function actionGalleryIsLoaded(state) {
  return new Promise((resolve, _reject) => {
    const newState = cloneState(state);
    newState.galleryLoaded = true;
    resolveAsCurrentState(resolve, newState);
  });
}

function actionShuffleGeneration(state, { rng }) {
  return new Promise((resolve, reject) => {
    const newState = cloneState(state);
    const prev = newState.previouslySelectedGenotypes;

    if (prev.length === 0) {
      actionInitialGeneration(newState).then(s => {
        resolveAsCurrentState(resolve, s);
      }).catch(error1 => {
        // handle error
        console.log(`worker: error of ${error1}`);
        reject(error1);
      });
    } else {
      Job.request(jobNewGeneration, {
        genotypes: prev,
        populationSize: newState.populationSize,
        traits: newState.traits,
        mutationRate: newState.mutationRate,
        rng
      }).then(({ genotypes }) => {
        newState.genotypes = genotypes;
        newState.selectedIndices = [];
        resolveAsCurrentState(resolve, newState);
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
    const newState = cloneState(state);
    const pg = newState.genotypes;
    const selectedIndices = newState.selectedIndices;
    const selectedGenos = [];
    for (let i = 0; i < selectedIndices.length; i++) {
      selectedGenos.push(pg[selectedIndices[i]]);
    }

    Job.request(jobNewGeneration, {
      genotypes: selectedGenos,
      populationSize: newState.populationSize,
      traits: newState.traits,
      mutationRate: newState.mutationRate,
      rng
    }).then(({ genotypes }) => {
      const previouslySelectedGenotypes =
            genotypes.slice(0, selectedIndices.length);

      newState.genotypes = genotypes;
      newState.previouslySelectedGenotypes = previouslySelectedGenotypes;
      newState.selectedIndices = [];

      resolveAsCurrentState(resolve, newState);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function wrapInPromise(state) {
  return new Promise((resolve, _reject) => {
    resolveAsCurrentState(resolve, state);
  });
}

function logMode(mode) {
  let name = '';
  switch (mode) {
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

export function createInitialState() {
  return {
    // the resolution of the high res image
    highResolution: [4096, 4096],
    placeholder: 'img/spinner.gif',
    populationSize: 24,
    mutationRate: 0.1,

    currentMode: SeniMode.gallery,
    galleryLoaded: false,
    previouslySelectedGenotypes: [],
    selectedIndices: [],
    scriptId: undefined,
    script: undefined,
    scriptHash: undefined,
    genotypes: [],
    traits: []
  };
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
    case 'SET_SCRIPT_ID':
      return actionSetScriptId(state, action);
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
    case 'GALLERY_LOADED':
      return actionGalleryIsLoaded(state, action);
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
