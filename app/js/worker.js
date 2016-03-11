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

import Bind from './lang/Bind';
import Runtime from './lang/Runtime';
import ProxyRenderer from './seni/ProxyRenderer';
import Genetic from './lang/Genetic';

const gProxyRenderer = new ProxyRenderer();
const gEnv = Bind.addBindings(Runtime.createEnv(), gProxyRenderer);
let gScriptHash = '';
let gFrontAst = undefined;
let gBackAst = undefined;
let gGenotype = undefined;

// todo: return more informative errors
function updateState(script, scriptHash, genotype) {
  if (scriptHash !== gScriptHash) {
    gScriptHash = scriptHash;

    gFrontAst = Runtime.buildFrontAst(script);
    if (gFrontAst.error) {
      return false;
    }

    gBackAst = Runtime.compileBackAst(gFrontAst.nodes);
  }

  if (genotype !== undefined) {
    gGenotype = genotype;
  } else {
    const traits = Genetic.buildTraits(gBackAst);
    gGenotype = Genetic.createGenotypeFromInitialValues(traits);
  }
  return true;
}

function titleForScript(env, scriptHash) {
  // default the scriptTitle to scriptHash
  // (but replace with 'title' binding if it's defined in the script)
  let scriptTitle = scriptHash;
  if (env) {
    const titleBinding = env.get('title');
    if (titleBinding) {
      scriptTitle = titleBinding.binding;
    }
  }
  return scriptTitle;
}

function render({ script, scriptHash, genotype }) {
  gProxyRenderer.reset();
  updateState(script, scriptHash, Immutable.fromJS(genotype));

  const res = Runtime.evalAst(gEnv, gBackAst, gGenotype);
  if (res === undefined) {
    throw new Error('worker.js::render evalAst returned undefined');
  }

  const finalEnv = res[0];
  const title = titleForScript(finalEnv, scriptHash);
  const commandBuffer = gProxyRenderer.getCommandBuffer();

  return { title, commandBuffer };
}

function unparse({ script, scriptHash, genotype }) {
  updateState(script, scriptHash, Immutable.fromJS(genotype));

  const newScript = Runtime.unparse(gFrontAst.nodes, gGenotype);

  return { script: newScript };
}

// this isn't saving the intermediate ASTs, perhaps do so later?
function buildTraits({ script, scriptHash }) {
  if (scriptHash !== gScriptHash) {
    gScriptHash = scriptHash;
    gFrontAst = Runtime.buildFrontAst(script);

    if (gFrontAst.error) {
      // don't cache the current compilation state variables
      gScriptHash = undefined;
      throw new Error(`worker.js::buildTraits: ${gFrontAst.error}`);
    }
    gBackAst = Runtime.compileBackAst(gFrontAst.nodes);
  }

  const traits = Genetic.buildTraits(gBackAst);

  return { traits };
}


function createInitialGeneration({ populationSize, traits }) {
  const random = (new Date()).toGMTString();
  const genotypes = [];

  const initialGeno = Genetic.createGenotypeFromInitialValues(traits);
  genotypes.push(initialGeno.toJS());

  for (let i = 1; i < populationSize; i++) {
    const genotype = Genetic.createGenotypeFromTraits(traits, i + random);
    genotypes.push(genotype.toJS());
  }

  return { genotypes };
}

function newGeneration(data) {
  const {
    genotypes,
    populationSize,
    traits,
    mutationRate,
    rng
  } = data;

  const newGenotypes = Genetic.nextGeneration(Immutable.fromJS(genotypes),
                                              populationSize,
                                              mutationRate,
                                              traits,
                                              rng);

  return { genotypes: newGenotypes };
}

function generateHelp() {
  // create a hash of document objects
  const res = gEnv.reduce((a, v, k) => {
    a[k] = v.pb.doc;
    return a;
  }, {});

  return res;
}

function register(callback) {
  self.addEventListener('message', e => {
    try {
      const { type, data } = JSON.parse(e.data);
      const result = callback(type, data);
      self.postMessage(JSON.stringify([null, result]));
    } catch (error) {
      self.postMessage(JSON.stringify([{message: error.message}]));
    }
  });
}

register((type, data) => {
  switch (type) {
  case 'RENDER':
    return render(data);
  case 'UNPARSE':
    return unparse(data);
  case 'BUILD_TRAITS':
    return buildTraits(data);
  case 'INITIAL_GENERATION':
    return createInitialGeneration(data);
  case 'NEW_GENERATION':
    return newGeneration(data);
  case 'GENERATE_HELP':
    return generateHelp(data);
  default:
    // throw unknown type
    throw new Error(`worker.js: Unknown type: ${type}`);
  }
});
