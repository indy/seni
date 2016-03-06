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

import NodeType from './NodeType';
import Compiler from './Compiler';
import Interpreter from './Interpreter';
import Bind from './Bind';
import PseudoRandom from '../seni/PseudoRandom';
import Immutable from 'immutable';

const logToConsole = false;

// the node will be in backAst form
function buildTraitFromNode(node, genes) {
  if (node.alterable === true) {
    // expect a form in the parameterAST
    let simplifiedAst, initialValue;

    if (node.type === NodeType.LIST || node.type === NodeType.VECTOR) {
      // wrap the node in an array and pass into Compiler.compileWithGenotype
      const compiledNodes = Compiler.compileWithGenotype([node], null);
      initialValue = compiledNodes[0];
      // Note: a problem with this is that a form written like '[1 2]' will be
      // represented in initialValue as the simplified ast: 'list 1 2'
    } else {
      initialValue = node.value;
    }

    if (node.parameterAST.length) {
      simplifiedAst = Compiler.compileWithGenotype(node.parameterAST, null);
    } else {
      // this is to allow code like (+ 2 {2})
      // which should behave as if there were no curly brackets
      // todo: implement identity in this context
      simplifiedAst = [[`gen/identity`, {value: initialValue}]];
    }

    const gene = {initialValue, simplifiedAst};
    genes.push(gene);  // mutate the genes
  }

  if (node.type === NodeType.LIST || node.type === NodeType.VECTOR) {
    node.children.map(child => buildTraitFromNode(child, genes));
  }
}

function buildGeneFromTrait(trait, env) {
  const simplifiedAst = trait.simplifiedAst;
  // evaluate all of the forms, returning the final [env, result]
  /* eslint-disable arrow-body-style */
  const [_, result, _error] = simplifiedAst.reduce(([e, f, err], form) => {
    if (err != Interpreter.NO_ERROR) {
      // if there`s an error keep on passing it along
      return [e, f, err];
    } else {
      return Interpreter.evaluate(e, form);
    }
  }, [env, false, Interpreter.NO_ERROR]);
  /* eslint-enable arrow-body-style */

  // return { error, result };
  return result;
}

function randomCrossover(genotypeA, genotypeB, mutationRate, traits, env) {
  // todo: assert that both genotypes have the same length

  const crossoverIndex = Number.parseInt(Math.random() * genotypeA.size, 10);
  if (logToConsole) {
    console.log(`randomCrossover index:`, crossoverIndex, mutationRate);
  }

  const spliceA = genotypeA.slice(0, crossoverIndex);
  const spliceB = genotypeB.slice(crossoverIndex, genotypeB.size);

  const childGenotype = spliceA.concat(spliceB);

  let i;
  for (i = 0; i < genotypeA.size; i++) {
    if (Math.random() < mutationRate) {
      // mutate this trait
      if (logToConsole) {
        console.log(`mutating gene `, i);
      }
      childGenotype[i] = buildGeneFromTrait(traits[i], env);
    }
  }

  if (logToConsole) {
    console.log(`childGenotype `, childGenotype);
  }

  return Immutable.fromJS(childGenotype);
}

function buildEnv(rng) {
  return Bind.addBracketBindings(
    Bind.addClassicBindings(
      Bind.addSpecialDebugBindings(
        Bind.addSpecialBindings(
          Interpreter.getBasicEnv()))),
    rng);
}

const Genetic = {

  buildTraits: ast => {
    const traits = [];
    ast.map(node => buildTraitFromNode(node, traits));
    return traits;
  },

  createGenotypeFromInitialValues: traits =>
    Immutable.fromJS(traits.map(g => g.initialValue)),

  createGenotypeFromTraits: (traits, seed) => {
    const rng = PseudoRandom.buildUnsigned(seed);
    const env = buildEnv(rng);

    // env is the environment used to evaluate the bracketed forms
    const genotype = traits.map(trait => buildGeneFromTrait(trait, env));

    return Immutable.fromJS(genotype);
  },

  nextGeneration: (genotypes, populationSize, mutationRate, traits, seed) => {
    // a silly mod method for creating the latest generation
    let i;
    let newGenotypes = genotypes;
    const rng = PseudoRandom.buildUnsigned(seed);
    const env = buildEnv(rng);

    if (logToConsole) {
      console.log(`Genetic::nextGeneration`, {populationSize,
                                              mutationRate,
                                              seed,
                                              size: genotypes.size});
    }

    for (i = genotypes.size; i < populationSize; i++) {
      const idxA = Number.parseInt(Math.random() * genotypes.size, 10);
      let idxB = Number.parseInt(Math.random() * genotypes.size, 10);

      // try not to use the same genotype for both a and b
      const retryCount = 10;
      for (let retry = 0; retry < retryCount; retry++) {
        if (idxB === idxA) {
          idxB = Number.parseInt(Math.random() * genotypes.size, 10);
        } else {
          break;
        }
      }

      const genotypeA = genotypes.get(idxA);
      const genotypeB = genotypes.get(idxB);

      if (logToConsole) {
        console.log(`using genotype indices: `, idxA, idxB);
      }

      const child = randomCrossover(genotypeA, genotypeB,
                                    mutationRate, traits, env);

      newGenotypes = newGenotypes.push(child);
    }
    return newGenotypes;
  }
};

export default Genetic;
