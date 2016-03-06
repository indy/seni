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

import register from 'promise-worker/register';
import Immutable from 'immutable';

import Bind from './lang/Bind';
import Runtime from './lang/Runtime';
import ProxyRenderer from './seni/ProxyRenderer';
import Genetic from './lang/Genetic';

const gProxyRenderer = new ProxyRenderer();
const gEnv = createEnv(gProxyRenderer);
let gScriptHash = '';
let gFrontAst = undefined;
let gBackAst = undefined;
let gGenotype = undefined;

function createEnv(proxyRenderer) {
  // an immutable var containing the base env for all evaluations
  return Bind.addBindings(Runtime.createEnv(), proxyRenderer);
}

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

function render({ workerId, data }) {
  const { script, scriptHash, genotype } = data;

  gProxyRenderer.reset();

  updateState(script, scriptHash, Immutable.fromJS(genotype));

  const res = Runtime.evalAst(gEnv, gBackAst, gGenotype);
  if (res === undefined) {
    return { status: 'ERROR', workerId };
  }

  const finalEnv = res[0];
  const title = titleForScript(finalEnv, scriptHash);
  const commandBuffer = gProxyRenderer.getCommandBuffer();

  return {
    status: 'OK',
    workerId,
    data: { title, commandBuffer }
  };
}

function unparse({ workerId, data }) {
  const { script, scriptHash, genotype } = data;

  updateState(script, scriptHash, Immutable.fromJS(genotype));

  const newScript = Runtime.unparse(gFrontAst.nodes, gGenotype);

  return {
    status: 'OK',
    workerId,
    data: { script: newScript }
  };
}

// this isn't saving the intermediate ASTs, perhaps do so later?
function buildTraits({ workerId, data }) {
  const { script, scriptHash } = data;

  if (scriptHash !== gScriptHash) {
    gScriptHash = scriptHash;

    gFrontAst = Runtime.buildFrontAst(script);
    if (gFrontAst.error) {
      return { status: 'ERROR' };
    }

    gBackAst = Runtime.compileBackAst(gFrontAst.nodes);
  }

  const traits = Genetic.buildTraits(gBackAst);

  return {
    status: 'OK',
    workerId,
    data: { traits }
  };
}


function createInitialGeneration({ workerId, data }) {
  const { populationSize, traits } = data;

  const random = (new Date()).toGMTString();
  const genotypes = [];

  const initialGeno = Genetic.createGenotypeFromInitialValues(traits);
  genotypes.push(initialGeno.toJS());

  for (let i = 1; i < populationSize; i++) {
    const genotype = Genetic.createGenotypeFromTraits(traits, i + random);
    genotypes.push(genotype.toJS());
  }

  return {
    status: 'OK',
    workerId,
    data: { genotypes }
  };
}

function newGeneration({ workerId, data }) {
  const {
    genotypes,
    populationSize,
    traits,
    mutationRate,
    rng
  } = data;

  const geno = Genetic.nextGeneration(Immutable.fromJS(genotypes),
                                      populationSize,
                                      mutationRate,
                                      traits,
                                      rng);

  return {
    status: 'OK',
    workerId,
    data: { genotypes: geno }
  };
}

function generateHelp(_args) {
}

register(args => {
  const { type } = args;

  switch (type) {
  case 'RENDER':
    return render(args);
  case 'UNPARSE':
    return unparse(args);
  case 'BUILD_TRAITS':
    return buildTraits(args);
  case 'INITIAL_GENERATION':
    return createInitialGeneration(args);
  case 'NEW_GENERATION':
    return newGeneration(args);
  case 'GENERATE_HELP':
    return generateHelp(args);
  default:
    return '';
  }
});
