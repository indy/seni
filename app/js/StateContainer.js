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

import { SeniMode } from './ui/SeniMode';
import Genetic from './lang/Genetic';
import Runtime from './lang/Runtime';

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

    frontAst: undefined,
    backAst: undefined,
    traits: undefined,

    saveState: createSaveState()
  });
}

function createSaveState() {
  return Immutable.fromJS({
    currentMode: SeniMode.gallery,
    previouslySelectedGenotypes: [],
    selectedIndices: [],
    script: undefined,
    genotypes: []
  });
}

export function createStore(initialState) {

  let currentState = initialState;

  function reducer(state, action) {
    switch (action.type) {
    case 'SET_MODE':
      return actionSetMode(state, action);
    case 'SET_SCRIPT':
      return actionSetScript(state, action);
    case 'SET_SELECTED_INDICES':
      return actionSetSelectedIndices(state, action);
    case 'INITIAL_GENERATION':
      return actionInitialGeneration(state, action);
    case 'NEXT_GENERATION':
      return actionNextGeneration(state, action);
    case 'SET_SAVE_STATE':
      return actionSetSaveState(state, action);
    case 'SET_PREVIOUSLY_SELECTED_GENOTYPES':
      return actionSetPreviouslySelectedGenotypes(state, action);
    case 'SETUP_AST_AND_TRAITS':
      return actionSetupAstAndTraits(state);
    default:
      return state;
    }
  }

  function getState() {
    return currentState;
  }

  function dispatch(action) {
    currentState = reducer(currentState, action);
  }

  return {
    getState,
    dispatch
  };
}

function actionSetMode(state, action) {
  return state.setIn(['saveState', 'currentMode'], action.mode);
}

function actionSetScript(state, action) {
  return state.setIn(['saveState', 'script'], action.script);
}

function actionSetSelectedIndices(state, action) {
  const si = action.selectedIndices || new Immutable.List();
  return state.setIn(['saveState', 'selectedIndices'], si);
}

function actionSetSaveState(state, action) {
  return state.set('saveState', action.saveState);
}

function actionSetPreviouslySelectedGenotypes(state, action) {
  return state.setIn(['saveState', 'previouslySelectedGenotypes'],
                     action.previouslySelectedGenotypes);
}

function actionSetupAstAndTraits(state) {
  const script = state.getIn(['saveState', 'script']);
  state = state.set('frontAst', Runtime.buildFrontAst(script));

  const frontAst = state.get('frontAst');
  state = state.set('backAst', Runtime.compileBackAst(frontAst));

  const backAst = state.get('backAst');
  state = state.set('traits', Genetic.buildTraits(backAst));

  return state;
}

// todo: should populationSize be passed in the action?
function actionInitialGeneration(state) {

  let genotype;
  const random = (new Date()).toGMTString();
  const traits = state.get('traits');
  const genotypes = [];
  const populationSize = state.get('populationSize');

  for (let i = 0; i < populationSize; i++) {
    if (i === 0) {
      genotype = Genetic.createGenotypeFromInitialValues(traits);
    } else {
      genotype = Genetic.createGenotypeFromTraits(traits, i + random);
    }
    genotypes.push(genotype);
  }

  const genos = new Immutable.List(genotypes);

  return state.setIn(['saveState', 'genotypes'], genos);
}

function actionNextGeneration(state, action) {
  const genotypes = Genetic.nextGeneration(action.genotypes,
                                           state.get('populationSize'),
                                           state.get('mutationRate'),
                                           state.get('traits'),
                                           action.rng);
  return state.setIn(['saveState', 'genotypes'], genotypes);
}
