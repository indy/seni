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
import Runtime from './lang/Runtime';
import Genetic from './lang/Genetic';

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
      return actionInitialGeneration(state);
    case 'NEXT_GENERATION':
      return actionNextGeneration(state, action);
    case 'SHUFFLE_GENERATION':
      return actionShuffleGeneration(state, action);
    case 'SET_STATE':
      return action.state;
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

function actionSetMode(state, { mode }) {
  return state.set('currentMode', mode);
}

function actionSetScript(state, { script }) {
  return state
    .set('script', script)
    .set('scriptHash', Util.hashCode(script));
}

function actionSetSelectedIndices(state, { selectedIndices }) {
  const si = selectedIndices || new Immutable.List();
  return state.set('selectedIndices', si);
}

// todo: should populationSize be passed in the action?
function actionInitialGeneration(state) {

  const script = state.get('script');
  const traits = buildTraits(script);

  let genotype;
  const random = (new Date()).toGMTString();
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

  return state
    .set('genotypes', new Immutable.List(genotypes))
    .set('previouslySelectedGenotypes', new Immutable.List())
    .set('selectedIndices', new Immutable.List());
}

function actionShuffleGeneration(state, { rng }) {

  const prev = state.get('previouslySelectedGenotypes');

  if (prev.size === 0) {
    return actionInitialGeneration(state);
  }

  const script = state.get('script');
  const traits = buildTraits(script);
  const genotypes = Genetic.nextGeneration(prev,
                                           state.get('populationSize'),
                                           state.get('mutationRate'),
                                           traits,
                                           rng);
  return state
    .set('genotypes', genotypes)
    .set('selectedIndices', new Immutable.List());
}

function actionNextGeneration(state, { rng }) {


  const pg = state.get('genotypes');
  const selectedIndices = state.get('selectedIndices');
  let selectedGenos = new Immutable.List();
  for (let i = 0; i < selectedIndices.size; i++) {
    selectedGenos = selectedGenos.push(pg.get(selectedIndices.get(i)));
  }

  const script = state.get('script');
  const traits = buildTraits(script);

  const genotypes = Genetic.nextGeneration(selectedGenos,
                                           state.get('populationSize'),
                                           state.get('mutationRate'),
                                           traits,
                                           rng);

  const previouslySelectedGenotypes = genotypes.slice(0, selectedIndices.size);

  return state.set('genotypes', genotypes)
    .set('previouslySelectedGenotypes', previouslySelectedGenotypes)
    .set('selectedIndices', new Immutable.List());
}


function buildTraits(script) {
  const frontAst = Runtime.buildFrontAst(script);
  const backAst = Runtime.compileBackAst(frontAst);
  const traits = Genetic.buildTraits(backAst);

  return traits;
}
