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

import Immutable from 'immutable';

import Util from './seni/Util';
import { SeniMode } from './ui/SeniMode';
import Workers from './workers';

let currentState = undefined;

function actionSetMode(state, { mode }) {
  return new Promise((resolve, _reject) => {
    currentState = state.set('currentMode', mode);
    resolve(currentState);
  });
}

function actionSetScript(state, { script }) {
  return new Promise((resolve, reject) => {
    currentState = state
      .set('script', script)
      .set('scriptHash', Util.hashCode(script));

    Workers.perform('BUILD_TRAITS', {
      script: currentState.get('script'),
      scriptHash: currentState.get('scriptHash')
    }).then(({ traits }) => {
      currentState = currentState.set('traits', traits);
      resolve(currentState);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function actionSetSelectedIndices(state, { selectedIndices }) {
  return new Promise((resolve, _reject) => {
    const si = selectedIndices || new Immutable.List();
    currentState = state.set('selectedIndices', si);
    resolve(currentState);
  });
}

// todo: should populationSize be passed in the action?
function actionInitialGeneration(state) {
  return new Promise((resolve, reject) => {
    Workers.perform('INITIAL_GENERATION', {
      traits: state.get('traits'),
      populationSize: state.get('populationSize')
    }).then(({ genotypes }) => {
      const im = Immutable.fromJS(genotypes);

      currentState = state
        .set('genotypes', new Immutable.List(im))
        .set('previouslySelectedGenotypes', new Immutable.List())
        .set('selectedIndices', new Immutable.List());

      resolve(currentState);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function actionShuffleGeneration(state, { rng }) {
  return new Promise((resolve, reject) => {
    const prev = state.get('previouslySelectedGenotypes');

    if (prev.size === 0) {
      actionInitialGeneration(state).then(s => {
        resolve(s);
      });
    } else {
      Workers.perform('NEW_GENERATION', {
        genotypes: prev.toJS(),
        populationSize: state.get('populationSize'),
        traits: state.get('traits'),
        mutationRate: state.get('mutationRate'),
        rng
      }).then(({ genotypes }) => {
        const im = Immutable.fromJS(genotypes);

        currentState = state
          .set('genotypes', new Immutable.List(im))
          .set('selectedIndices', new Immutable.List());

        resolve(currentState);
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
    const pg = state.get('genotypes');
    const selectedIndices = state.get('selectedIndices');
    let selectedGenos = new Immutable.List();
    for (let i = 0; i < selectedIndices.size; i++) {
      selectedGenos = selectedGenos.push(pg.get(selectedIndices.get(i)));
    }

    Workers.perform('NEW_GENERATION', {
      genotypes: selectedGenos.toJS(),
      populationSize: state.get('populationSize'),
      traits: state.get('traits'),
      mutationRate: state.get('mutationRate'),
      rng
    }).then(({ genotypes }) => {
      const im = Immutable.fromJS(genotypes);
      const previouslySelectedGenotypes = im.slice(0, selectedIndices.size);

      currentState = state
        .set('genotypes', im)
        .set('previouslySelectedGenotypes', previouslySelectedGenotypes)
        .set('selectedIndices', new Immutable.List());

      resolve(currentState);
    }).catch(error => {
      // handle error
      console.log(`worker: error of ${error}`);
      reject(error);
    });
  });
}

function wrapInPromise(state) {
  return new Promise((resolve, _reject) => {
    currentState = state;
    resolve(currentState);
  });
}

/**
 * Creates the immutable SeniState
 *
 * @private
 * @returns {Immutable Map} a basic SeniState with a valid renderer and env
 */
export function createInitialState() {
  return Immutable.fromJS({
    // the resolution of the high res image
    highResolution: [2048, 2048],
    placeholder: 'img/spinner.gif',
    populationSize: 24,
    mutationRate: 0.1,

    currentMode: SeniMode.gallery,
    previouslySelectedGenotypes: [],
    selectedIndices: [],
    script: undefined,
    scriptHash: undefined,
    genotypes: [],
    traits: []
  });
}

export function createStore(initialState) {
  currentState = initialState;

  function reducer(state, action) {
    switch (action.type) {
    case 'SET_MODE':
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
      return wrapInPromise(action.state);
    default:
      return wrapInPromise(state);
    }
  }

  function getState() {
    return currentState;
  }

  function dispatch(action) {
    return reducer(currentState, action);
  }

  return {
    getState,
    dispatch
  };
}
